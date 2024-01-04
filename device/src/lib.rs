use std::collections::HashMap;
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

/// A `Device` exists on the network and is discoverable via mDNS.
///
/// **Design Decision**: `Device` must implement `Sized` so that we can call `Self::new` in the
/// `start` methods of `Actuator`, `Controller`, `Environment`, and `Sensor`. We need a `new` method
/// on `Device` because we want to be able to construct a new instance of the `Device` within a new
/// thread, without the user having to do all of this setup correctly.
pub trait Device: Sized {
    /// Returns the user-defined name of this `Device`.
    fn get_name(&self) -> &Name;

    /// Returns the unique ID of this `Device`.
    fn get_id(&self) -> &Id;

    /// Returns the model of this `Device`, which may or may not be supported by the `Controller`.
    fn get_model() -> Model;

    /// Returns the helper which defines how to handle HTTP requests.
    fn get_handler(&self) -> Handler;

    /// Provides a standard way to deal with failures in `get_handler()`.
    fn handler_failure(self_name: Name, stream: &mut TcpStream, msg: &str) {
        println!("[{}] {}", self_name, msg);
        let response = Message::respond_bad_request().with_body(msg);
        response.write(stream)
    }

    /// Registers this `Device` with mDNS in the specified group.
    fn register(&self, ip: IpAddr, port: u16, group: &str, mdns: ServiceDaemon) {
        let host = ip.to_string();
        let label = self.get_name().to_string();
        let name = format!("{}.{}", self.get_id(), Self::get_model());
        let domain = format!("{}._tcp.local.", group);

        println!("[Device::register] registering new Device \"{}\" via mDNS at {}.{}", label, name, domain);

        let mut properties = HashMap::new();
        properties.insert("id".to_string(), self.get_id().to_string());
        properties.insert("name".to_string(), self.get_name().to_string());
        properties.insert("model".to_string(), Self::get_model().to_string());

        let my_service = ServiceInfo::new(domain.as_str(), name.as_str(), host.as_str(), ip, port, properties).unwrap();

        mdns.register(my_service).unwrap()
    }

    /// Creates a `TcpListener` and binds it to the specified `ip` and `port`.
    fn bind(&self, address: Address) -> TcpListener {
        let address = address.to_string();
        let name = &self.get_name();

        println!("[Device::bind] binding new TCP listener to \"{}\" at {}", name, address);

        TcpListener::bind(address).unwrap()
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

    /// Extracts the `Address` of a `Device` from its `ServiceInfo` found via mDNS.
    fn extract_address(info: &ServiceInfo) -> Address {
        let ip = *info.get_addresses().iter().next().unwrap();
        let port = info.get_port();
        Address::new(ip, port)
    }

    /// Extracts the [`Id`](Id) of a `Device` from its `ServiceInfo`.
    ///
    /// The `id` property is set when a device is [`register`ed](Self::register) with mDNS.
    fn extract_id(info: &ServiceInfo) -> Option<Id> {
        let id = info.get_property("id").map(|p| p.to_string());
        id.map(|i| Id::new(i.trim_start_matches("id=")))
    }

    /// Extracts the [`Model`](Model) of a `Device` from its `ServiceInfo`.
    ///
    /// The `model` property is set when a device is [`register`ed](Self::register) with mDNS.
    fn extract_model(info: &ServiceInfo) -> Option<Result<Model, String>> {
        let model = info.get_property("model").map(|p| p.to_string());
        model.map(|m| Model::parse(m.trim_start_matches("model=")))
    }

    /// Extracts the [`Name`](Name) of a `Device` from its `ServiceInfo`.
    ///
    /// The `name` property is set when a device is [`register`ed](Self::register) with mDNS.
    fn extract_name(info: &ServiceInfo) -> Option<Name> {
        let name = info.get_property("name").map(|p| p.to_string());
        name.map(|i| Name::new(i.trim_start_matches("name=")))
    }

    /// Creates a new thread to continually discover `Device`s on the network in the specified group.
    fn discover_continually(&self, group: &str, devices: &Arc<Mutex<HashMap<Id, ServiceInfo>>>, mdns: ServiceDaemon) -> JoinHandle<()> {
        let group = String::from(group);
        let mutex = Arc::clone(devices);

        // Anything which depends on self must be cloned outside of the || lambda.
        // We cannot refer to `self` inside of this lambda.
        let self_name = self.get_name().to_string();

        std::thread::spawn(move || {
            let service_type = format!("{}._tcp.local.", group);
            let receiver = mdns.browse(service_type.as_str()).unwrap();

            while let Ok(event) = receiver.recv() {
                if let mdns_sd::ServiceEvent::ServiceResolved(info) = event {
                    let id = Self::extract_id(&info);
                    let devices_lock = mutex.lock();
                    let mut devices_guard = devices_lock.unwrap();

                    println!(
                        "[Device::discover_continually] \"{}\" discovered \"{}\"",
                        self_name,
                        info.get_property("name").map(|p| p.val_str()).unwrap_or("<unknown>")
                    );

                    id.map(|i| devices_guard.insert(i, info));
                }
            }
        })
    }

    /// Creates a new thread to discover a single `Device` on the network in the specified `group`.
    ///
    /// Once that single `Device` is discovered, the thread is completed.
    fn discover_once(&self, group: &str, devices: &Arc<Mutex<Option<ServiceInfo>>>, mdns: ServiceDaemon) -> JoinHandle<()> {
        let group = String::from(group);
        let mutex = Arc::clone(devices);

        // Anything which depends on self must be cloned outside of the || lambda.
        // We cannot refer to `self` inside of this lambda.
        let self_name = self.get_name().to_string();

        std::thread::spawn(move || {
            let service_type = format!("{}._tcp.local.", group);
            let receiver = mdns.browse(service_type.as_str()).unwrap();

            while let Ok(event) = receiver.recv() {
                if let mdns_sd::ServiceEvent::ServiceResolved(info) = event {
                    let devices_lock = mutex.lock();
                    let mut device = devices_lock.unwrap();

                    println!(
                        "[Device::discover_once] \"{}\" discovered \"{}\"",
                        self_name,
                        info.get_property("name").map(|p| p.val_str()).unwrap_or("<unknown>")
                    );

                    let _ = device.insert(info);
                    break;
                }
            }
        })
    }
}
