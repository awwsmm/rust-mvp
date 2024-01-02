use std::collections::HashMap;
use std::io::BufReader;
use std::net::{IpAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use mdns_sd::{ServiceDaemon, ServiceInfo};

use crate::address::Address;
use crate::id::Id;
use crate::message::Message;
use crate::model::Model;
use crate::name::Name;

pub mod address;
pub mod id;
pub mod message;
pub mod model;
pub mod name;

/// A `Handler` describes how a `Device` should handle incoming HTTP requests.
pub type Handler = Box<dyn Fn(&mut TcpStream)>;

/// A _target_ is a `Device` to which _this_ `Device` _sends_ `Message`s.
///
/// A `Device` can be uniquely identified by its `Id`, but to send it a message,
/// we also need its `Address`.
pub type Targets = Arc<Mutex<HashMap<Id, ServiceInfo>>>;

/// A `Device` exists on the network and is discoverable via mDNS.
pub trait Device: Sized {
    /// Returns the user-friendly name of this `Device`.
    fn get_name(&self) -> &Name;

    /// Returns the unique ID of this `Device`.
    fn get_id(&self) -> &Id;

    /// Returns the model of this `Device`, which may or may not be supported by the `Controller`.
    fn get_model() -> Model;

    /// Returns the ip:port address of this `Device` (e.g. "192.168.1.251:8787').
    fn get_address(&self) -> Address;

    /// Returns the helper which defines how to handle HTTP requests.
    fn get_handler(&self) -> Handler;

    /// Registers this `Device` with mDNS in the specified group.
    fn register(&self, ip: IpAddr, port: u16, group: &str, mdns: ServiceDaemon) {
        let host = ip.to_string();
        let label = self.get_name().to_string();
        let name = format!("{}.{}", self.get_id(), Self::get_model());
        let domain = format!("{}._tcp.local.", group);

        println!(
            "[Device::register] registering new Device \"{}\" via mDNS at {}.{}",
            label, name, domain
        );

        let mut properties = HashMap::new();
        properties.insert("id".to_string(), self.get_id().to_string());
        properties.insert("name".to_string(), self.get_name().to_string());
        properties.insert("model".to_string(), Self::get_model().to_string());

        let my_service = ServiceInfo::new(
            domain.as_str(),
            name.as_str(),
            host.as_str(),
            ip,
            port,
            properties,
        )
        .unwrap();

        mdns.register(my_service).unwrap()
    }

    /// Extracts the `Address` of a `Device` from its `ServiceInfo` found via mDNS.
    fn extract_address(info: &ServiceInfo) -> Address {
        let ip = *info.get_addresses().iter().next().unwrap();
        let port = info.get_port();
        Address::new(ip, port)
    }

    /// Creates a `TcpListener` and binds it to the specified `ip` and `port`.
    fn bind(&self, address: Address) -> TcpListener {
        let address = address.to_string();
        let name = &self.get_name();

        println!(
            "[Device::bind] binding new TCP listener to \"{}\" at {}",
            name, address
        );

        TcpListener::bind(address).unwrap()
    }

    /// Reads an HTTP request from a `TcpStream` and parses it into the request line, headers, and
    /// body; then responds with a `200 OK` ACK to close the socket.
    ///
    /// **Design Decision**: in this codebase, `Message`s are HTTP requests. All communication happens asynchronously via
    /// "fire and forget" HTTP requests (all responses to all messages are "200 OK"). This _asynchronous message-passing_
    /// style of communication is the de-facto standard in
    /// [microservices design](https://docs.aws.amazon.com/whitepapers/latest/microservices-on-aws/asynchronous-messaging-and-event-passing.html).
    fn ack_and_parse_request(
        sender_name: &str,
        sender_address: Address,
        mut stream: &mut TcpStream,
    ) -> Result<Message, String> {
        let request = Message::read(BufReader::new(&mut stream));

        // every HTTP request gets a 200 OK "ack" to close the HTTP socket
        let response = Message::ack(sender_name, sender_address);
        response.write(stream);

        request
    }

    /// `register`s and `bind`s this `Device`, then spawns a new thread where it will continually
    /// listen for incoming `TcpStream`s and handle them appropriately.
    fn respond(&self, ip: IpAddr, port: u16, group: &str, mdns: ServiceDaemon) {
        self.register(ip, port, group, mdns);
        let listener = self.bind(Address::new(ip, port));

        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            (*self.get_handler())(&mut stream);
        }
    }

    /// Extracts the [`Id`](crate::Id) of a `Device` from its `ServiceInfo`.
    ///
    /// The `id` property is set when a device is [`register`ed](Self::register) with mDNS.
    fn extract_id(info: &ServiceInfo) -> Option<Id> {
        let id = info.get_property("id").map(|p| p.to_string());
        id.map(|i| Id::new(i.trim_start_matches("id=")))
    }

    /// Extracts the [`Model`](crate::Model) of a `Device` from its `ServiceInfo`.
    ///
    /// The `model` property is set when a device is [`register`ed](Self::register) with mDNS.
    fn extract_model(info: &ServiceInfo) -> Option<Result<Model, String>> {
        let model = info.get_property("model").map(|p| p.to_string());
        model.map(|m| Model::parse(m.trim_start_matches("model=")))
    }

    /// Creates a new thread to continually discover `Device`s on the network in the specified group.
    fn discover(&self, group: &str, devices: &Targets, mdns: ServiceDaemon) -> JoinHandle<()> {
        let group = String::from(group);

        // clone the Arc<Mutex<>> around the devices so we can update them in multiple threads
        let devices_mutex = Arc::clone(devices);
        let self_name = self.get_name().to_string();

        std::thread::spawn(move || {
            let service_type = format!("{}._tcp.local.", group);
            let receiver = mdns.browse(service_type.as_str()).unwrap();

            while let Ok(event) = receiver.recv() {
                if let mdns_sd::ServiceEvent::ServiceResolved(info) = event {
                    let id = Self::extract_id(&info);
                    let devices_lock = devices_mutex.lock();
                    let mut devices_guard = devices_lock.unwrap();

                    println!(
                        "[Device::discover] \"{}\" discovered \"{}\"",
                        self_name,
                        info.get_property("name")
                            .map(|p| p.val_str())
                            .unwrap_or("<unknown>")
                    );

                    id.map(|i| devices_guard.insert(i, info));
                }
            }
        })
    }

    fn targets_by_group(&self) -> HashMap<String, Targets>;

    fn new(id: Id, name: Name, address: Address) -> Self;
}
