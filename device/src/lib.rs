use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::net::{IpAddr, TcpListener, TcpStream};
use std::thread::JoinHandle;

use mdns_sd::ServiceInfo;

use crate::handler::Handler;
use crate::id::Id;
use crate::model::Model;
use crate::name::Name;

pub mod handler;
pub mod id;
pub mod model;
pub mod name;

/// A `Device` exists on the network and is discoverable via mDNS.
pub trait Device {
    /// Returns the user-friendly name of this `Device`.
    fn get_name(&self) -> &Name;

    /// Returns the model of this `Device`, which may or may not be supported by the `Controller`.
    fn get_model(&self) -> &Model;

    /// Returns the unique ID of this `Device`.
    fn get_id(&self) -> &Id;

    /// Registers this `Device` with mDNS in the specified group.
    fn register(&self, ip: IpAddr, port: u16, group: &str) {
        let mdns = mdns_sd::ServiceDaemon::new().unwrap();
        let host = ip.clone().to_string();
        let name = self.get_name();
        let domain = format!("{}._tcp.local.", group);

        println!("Registering new device via mDNS at {}.{}", name, domain);

        let mut properties = HashMap::new();
        properties.insert(String::from("id"), self.get_id().to_string());
        properties.insert(String::from("model"), self.get_model().to_string());

        let my_service = ServiceInfo::new(
            domain.as_str(),
            name.0.as_str(),
            host.as_str(),
            ip,
            port,
            properties,
        )
        .unwrap();

        mdns.register(my_service).unwrap()
    }

    /// Creates a `TcpListener` and binds it to the specified `ip` and `port`.
    fn bind(&self, ip: IpAddr, port: u16) -> TcpListener {
        let host = ip.clone().to_string();
        let address = format!("{}:{}", host, port);
        let name = &self.get_name();

        println!("Creating new device '{}' at {}", name, address);

        TcpListener::bind(address).unwrap()
    }

    fn get_handler() -> Handler;

    fn extract_request(stream: &TcpStream) -> String {
        let mut request = String::new();
        BufReader::new(stream).read_line(&mut request).unwrap();
        request
    }

    /// `register`s and `bind`s this `Device`, then spawns a new thread where it will continually
    /// listen for incoming `TcpStream`s and handles them appropriately.
    fn run(&self, ip: IpAddr, port: u16, group: &str) -> JoinHandle<()> {
        self.register(ip, port, group);
        let listener = self.bind(ip, port);
        let handler = Self::get_handler();

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                (handler.handle)(&mut stream);
            }
        })
    }
}
