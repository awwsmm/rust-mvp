use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use mdns_sd::{ServiceDaemon, ServiceInfo};

use datum::kind::Kind;
use datum::unit::Unit;
use device::address::Address;
use device::id::Id;
use device::name::Name;
use device::Device;

/// A Sensor collects data from the Environment.
pub trait Sensor: Device {
    fn new(id: Id, name: Name, address: Address) -> Self;

    fn get_environment(&self) -> &Arc<Mutex<Option<ServiceInfo>>>;

    fn get_controller(&self) -> &Arc<Mutex<Option<ServiceInfo>>>;

    fn get_environment_info(&self) -> Option<ServiceInfo>;

    fn get_controller_info(&self) -> Option<ServiceInfo>;

    fn get_datum_value_type(&self) -> Kind;

    fn get_datum_unit(&self) -> Unit;

    fn start(ip: IpAddr, port: u16, id: Id, name: Name, group: String) -> JoinHandle<()> {
        std::thread::spawn(move || {
            let device = Self::new(id, name, Address::new(ip, port));

            let mdns = ServiceDaemon::new().unwrap();

            device.discover_once("_controller", device.get_controller(), mdns.clone());
            device.discover_once("_environment", device.get_environment(), mdns.clone());

            device.respond(ip, port, group.as_str(), mdns)
        })
    }
}
