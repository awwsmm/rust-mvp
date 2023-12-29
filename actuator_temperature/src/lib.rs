use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use mdns_sd::{ServiceDaemon, ServiceInfo};

use actuator::Actuator;
use device::id::Id;
use device::model::Model;
use device::name::Name;
use device::{Device, Handler};

pub mod command;

pub struct TemperatureActuator {
    id: Id,
    name: Name,
    pub env: Arc<Mutex<HashMap<Id, ServiceInfo>>>,
    address: String,
}

impl Device for TemperatureActuator {
    fn get_name(&self) -> &Name {
        &self.name
    }

    fn get_id(&self) -> &Id {
        &self.id
    }

    fn get_model() -> Model {
        Model::Thermo5000
    }

    fn get_group() -> String {
        <Self as Actuator>::get_group()
    }

    fn get_address(&self) -> &String {
        &self.address
    }

    fn get_handler(&self) -> Handler {
        self.default_handler()
    }

    fn start(
        ip: IpAddr,
        port: u16,
        id: Id,
        name: Name,
        mdns: Arc<ServiceDaemon>,
    ) -> JoinHandle<()> {
        let host = ip.clone().to_string();
        let address = <Self as Device>::address(host, port.to_string());

        std::thread::spawn(move || {
            println!(">>> [actuator_temp start] SPAWNED A NEW THREAD");

            let device = Self::new(id, name, address);

            let mut targets = HashMap::new();
            targets.insert("_environment".into(), &device.env);

            device.run(ip, port, "_actuator", targets, mdns);
        })
    }
}

impl Actuator for TemperatureActuator {
    fn get_environment(&self) -> Option<ServiceInfo> {
        let lock = self.env.lock();
        let guard = lock.unwrap();

        println!("!!! env: {:?}", guard.keys());

        guard.get(&Id::new("environment")).cloned()
    }
}

impl TemperatureActuator {
    pub fn new(id: Id, name: Name, address: String) -> Self {
        Self {
            id,
            name,
            env: Arc::new(Mutex::new(HashMap::new())),
            address,
        }
    }
}
