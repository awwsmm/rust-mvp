use std::collections::HashMap;
use std::io::Write;
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
    ///
    /// **Design Decision**: `tcp_stream` is of type `impl Write` rather than `TcpStream` because
    /// this is easier to test below. We do not use any `TcpStream`-specific APIs in this method.
    fn handler_failure(self_name: Name, tcp_stream: &mut impl Write, msg: &str) {
        println!("[{}] {}", self_name, msg);
        let response = Message::respond_bad_request().with_body(msg);
        response.write(tcp_stream)
    }

    /// Returns the `ServiceInfo` for this `Device`, which is used to register it via mDNS.
    ///
    /// **Design Decision**: this logic has been extracted from [`register`](Self::register) to make
    /// it easier to test (no `mdns: ServiceDaemon` is required).
    fn get_service_info(&self, ip: IpAddr, port: u16, group: &str) -> ServiceInfo {
        let host = ip.to_string();
        let label = self.get_name().to_string();
        let name = format!("{}.{}", self.get_id(), Self::get_model());
        let domain = format!("{}._tcp.local.", group);

        println!("[Device::register] registering new Device \"{}\" via mDNS at {}.{}", label, name, domain);

        let mut properties = HashMap::new();
        properties.insert("id".to_string(), self.get_id().to_string());
        properties.insert("name".to_string(), self.get_name().to_string());
        properties.insert("model".to_string(), Self::get_model().to_string());

        ServiceInfo::new(domain.as_str(), name.as_str(), host.as_str(), ip, port, properties).unwrap()
    }

    /// Registers this `Device` with mDNS in the specified group.
    // coverage: off
    // it is not possible to test this outside of an integration test which uses mDNS
    fn register(&self, service_info: ServiceInfo, mdns: ServiceDaemon) {
        mdns.register(service_info).unwrap()
    }
    // coverage: on

    /// Creates a `TcpListener` and binds it to the specified `ip` and `port`.
    // coverage: off
    // it is not possible to test this without actually binding to the address
    fn bind(&self, address: Address) -> TcpListener {
        let address = address.to_string();
        let name = &self.get_name();

        println!("[Device::bind] binding new TCP listener to \"{}\" at {}", name, address);

        TcpListener::bind(address).unwrap()
    }
    // coverage: on

    /// `register`s and `bind`s this `Device`, then spawns a new thread where it will continually
    /// listen for incoming `TcpStream`s and handle them appropriately.
    // coverage: off
    // it is not possible to test this outside of an integration test
    fn respond(&self, ip: IpAddr, port: u16, group: &str, mdns: ServiceDaemon) {
        let service_info = self.get_service_info(ip, port, group);
        self.register(service_info, mdns);
        let listener = self.bind(Address::new(ip, port));

        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            (*self.get_handler())(&mut stream);
        }
    }
    // coverage: on

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

    /// Creates a new thread to discover one or more `Device`s on the network in the specified `group`.
    fn discover<T: Sync + Send + 'static>(
        &self,
        group: &str,
        container: &Arc<Mutex<T>>,
        mdns: ServiceDaemon,
        save: fn(ServiceInfo, &String, &Arc<Mutex<T>>),
        unique: bool,
    ) -> JoinHandle<()> {
        let group = String::from(group);
        let mutex = Arc::clone(container);

        // Anything which depends on self must be cloned outside of the || lambda.
        // We cannot refer to `self` inside of this lambda.
        let self_name = self.get_name().to_string();

        std::thread::spawn(move || {
            let service_type = format!("{}._tcp.local.", group);
            let receiver = mdns.browse(service_type.as_str()).unwrap();

            while let Ok(event) = receiver.recv() {
                if let mdns_sd::ServiceEvent::ServiceResolved(info) = event {
                    save(info, &self_name, &mutex);
                    if unique {
                        break;
                    }
                }
            }
        })
    }

    /// Creates a new thread to discover a single `Device` on the network in the specified `group`.
    ///
    /// Once that single `Device` is discovered, the thread is completed.
    fn discover_once(&self, group: &str, devices: &Arc<Mutex<Option<ServiceInfo>>>, mdns: ServiceDaemon) -> JoinHandle<()> {
        self.discover(group, devices, mdns, Self::save_unique_device, true)
    }

    /// Creates a new thread to continually discover `Device`s on the network in the specified group.
    fn discover_continually(&self, group: &str, devices: &Arc<Mutex<HashMap<Id, ServiceInfo>>>, mdns: ServiceDaemon) -> JoinHandle<()> {
        self.discover(group, devices, mdns, Self::save_device, false)
    }

    /// Saves the `ServiceInfo` of a `Device` found via mDNS into the `map`.
    ///
    /// **Design Decision**: this logic has been extracted from
    /// [`discover_continually`](Self::discover_continually) to make it easier to test.
    fn save_device(info: ServiceInfo, self_name: &String, map: &Arc<Mutex<HashMap<Id, ServiceInfo>>>) {
        let id = Self::extract_id(&info);
        let devices_lock = map.lock();
        let mut devices_guard = devices_lock.unwrap();

        println!(
            "[Device::discover_continually] \"{}\" discovered \"{}\"",
            self_name,
            info.get_property("name").map(|p| p.val_str()).unwrap_or("<unknown>")
        );

        id.map(|i| devices_guard.insert(i, info));
    }

    /// Saves the `ServiceInfo` of a `Device` found via mDNS into the `container`.
    ///
    /// **Design Decision**: this logic has been extracted from
    /// [`discover_once`](Self::discover_once) to make it easier to test.
    fn save_unique_device(info: ServiceInfo, self_name: &String, container: &Arc<Mutex<Option<ServiceInfo>>>) {
        let devices_lock = container.lock();
        let mut device = devices_lock.unwrap();

        println!(
            "[Device::discover_once] \"{}\" discovered \"{}\"",
            self_name,
            info.get_property("name").map(|p| p.val_str()).unwrap_or("<unknown>")
        );

        let _ = device.insert(info);
    }
}
