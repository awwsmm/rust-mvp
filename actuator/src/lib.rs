use std::fmt::Display;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use mdns_sd::{ServiceDaemon, ServiceInfo};

use device::address::Address;
use device::id::Id;
use device::name::Name;
use device::Device;

/// An Actuator mutates the Environment.
pub trait Actuator: Device {
    fn new(id: Id, name: Name, address: Address) -> Self;

    fn get_environment(&self) -> &Arc<Mutex<Option<ServiceInfo>>>;

    fn get_environment_info(&self) -> Option<ServiceInfo>;

    fn start(ip: IpAddr, port: u16, id: Id, name: Name, group: String) -> JoinHandle<()> {
        std::thread::spawn(move || {
            let device = Self::new(id, name, Address::new(ip, port));

            let mdns = ServiceDaemon::new().unwrap();

            device.discover_once("_environment", device.get_environment(), mdns.clone());

            device.respond(ip, port, group.as_str(), mdns)
        })
    }
}

pub trait Command: Display {}
