use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::net::{IpAddr, TcpListener};

use mdns_sd::ServiceInfo;

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
    fn listener(&self, ip: IpAddr, port: u16) -> TcpListener {
        let host = ip.clone().to_string();
        let address = format!("{}:{}", host, port);
        let name = &self.get_name();

        println!("Creating new device '{}' at {}", name, address);

        TcpListener::bind(address).unwrap()
    }

    /// Registers this `Device` with mDNS in the specified `group` and binds it to listen at the specified `ip` and `port.
    fn bind(&self, ip: IpAddr, port: u16, group: &str) -> TcpListener {
        self.register(ip, port, group);
        self.listener(ip, port)
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Name(pub String);

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Name {
    pub fn new(name: &str) -> Name {
        Name(String::from(name))
    }
}

#[derive(PartialEq, Debug, Clone)]
pub struct Model(pub String);

impl Display for Model {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Model {
    pub fn new(model: &str) -> Model {
        Model(String::from(model))
    }
}

#[derive(PartialEq, Debug, Eq, Hash, Clone)]
pub struct Id(pub String);

impl Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Id {
    pub fn new(id: &str) -> Id {
        Id(String::from(id))
    }
}
