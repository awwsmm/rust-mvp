use std::collections::HashMap;
use std::io::BufReader;
use std::net::{IpAddr, TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use mdns_sd::{ServiceDaemon, ServiceInfo};

use crate::id::Id;
use crate::message::Message;
use crate::model::Model;
use crate::name::Name;

pub mod id;
pub mod message;
pub mod model;
pub mod name;

// We want to avoid blocking req-res loops and instead model
// all communication in a "fire and forget" manner. We never listen for responses.
// So every message gets responded to with an ACK (200 OK).
pub type Handler = Box<dyn Fn(&mut TcpStream)>;

/// A `Device` exists on the network and is discoverable via mDNS.
pub trait Device {
    /// Returns the user-friendly name of this `Device`.
    fn get_name(&self) -> &Name;

    /// Returns the unique ID of this `Device`.
    fn get_id(&self) -> &Id;

    /// Returns the model of this `Device`, which may or may not be supported by the `Controller`.
    fn get_model() -> Model;

    /// Returns the ip:port of this `Device` (e.g. "192.168.1.251:8787').
    fn get_address(&self) -> &String;

    /// Returns the helper which defines how to handle HTTP requests.
    fn get_handler(&self) -> Handler;

    /// Registers this `Device` with mDNS in the specified group.
    fn register(&self, ip: IpAddr, port: u16, group: &str, mdns: ServiceDaemon) {
        let host = ip.clone().to_string();
        let label = self.get_name().to_string();
        let name = format!("{}.{}", self.get_id(), Self::get_model());
        let domain = format!("{}._tcp.local.", group);

        println!(
            "[Device::register] registering new Device \"{}\" via mDNS at {}.{}",
            label, name, domain
        );

        let mut properties = HashMap::new();
        properties.insert(String::from("id"), self.get_id().to_string());
        properties.insert(String::from("name"), self.get_name().to_string());
        properties.insert(String::from("model"), Self::get_model().to_string());

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

    fn address(host: String, port: String) -> String {
        format!("{}:{}", host, port)
    }

    fn extract_address(info: &ServiceInfo) -> String {
        Self::address(
            info.get_hostname().trim_end_matches('.').to_string(),
            info.get_port().to_string(),
        )
    }

    /// Creates a `TcpListener` and binds it to the specified `ip` and `port`.
    fn bind(&self, ip: IpAddr, port: u16) -> TcpListener {
        let host = ip.clone().to_string();
        let address = Self::address(host, port.to_string());
        let name = &self.get_name();

        println!(
            "[Device::bind] binding new TCP listener to \"{}\" at {}",
            name, address
        );

        TcpListener::bind(address).unwrap()
    }

    /// Reads a message from a `TcpStream` and parses it into the message line, headers, and body.
    fn ack_and_parse_request(
        sender_name: String,
        sender_address: String,
        mut stream: &mut TcpStream,
    ) -> Result<Message, String> {
        let request = Message::from(BufReader::new(&mut stream));

        // every HTTP request gets a 200 OK "ack" to close the HTTP socket
        let response = Message::ack(sender_name, sender_address);
        response.send(stream);

        request
    }

    /// `register`s and `bind`s this `Device`, then spawns a new thread where it will continually
    /// listen for incoming `TcpStream`s and handles them appropriately.
    fn respond(&self, ip: IpAddr, port: u16, group: &str, mdns: ServiceDaemon) {
        self.register(ip, port, group, mdns.clone());
        let listener = self.bind(ip, port);

        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            (*self.get_handler())(&mut stream);
        }
    }

    /// Configures this `Device` to `respond` to incoming requests and discover `targets` for outgoing requests.
    fn run(
        &self,
        ip: IpAddr,
        port: u16,
        group: &str,
        targets: HashMap<String, &Arc<Mutex<HashMap<Id, ServiceInfo>>>>,
    ) {
        // each Device must have its own ServiceDaemon so multiple Devices can consume the same event
        // (i.e. the Environment coming online)
        let mdns = ServiceDaemon::new().unwrap();

        for (group, devices) in targets.iter() {
            self.discover(group, devices, mdns.clone());
        }
        self.respond(ip, port, group, mdns.clone())
    }

    fn extract_id(info: &ServiceInfo) -> Option<Id> {
        let id = info.get_property("id").map(|p| p.to_string());
        id.map(|i| Id::new(i.trim_start_matches("id=")))
    }

    fn extract_model(info: &ServiceInfo) -> Option<Result<Model, String>> {
        let model = info.get_property("model").map(|p| p.to_string());
        model.map(|m| Model::parse(m.trim_start_matches("model=")))
    }

    /// Creates a new thread to continually discover `Device`s on the network in the specified group.
    fn discover(
        &self,
        group: &str,
        devices: &Arc<Mutex<HashMap<Id, ServiceInfo>>>,
        mdns: ServiceDaemon,
    ) -> JoinHandle<()> {
        let group = String::from(group);

        // clone the Arc<Mutex<>> around the devices so we can update them in multiple threads
        let devices_mutex = Arc::clone(devices);
        let self_name = self.get_name().to_string();
        let mdns = mdns.clone();

        std::thread::spawn(move || {
            // let mdns = mdns_sd::ServiceDaemon::new().unwrap();
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

    fn start(ip: IpAddr, port: u16, id: Id, name: Name) -> JoinHandle<()>;
}
