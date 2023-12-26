use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};

use mdns_sd::ServiceInfo;

use actuator::Actuator;
use device::handler::Handler;
use device::id::Id;
use device::model::Model;
use device::name::Name;
use device::Device;

pub mod command;

pub struct TemperatureActuator {
    id: Id,
    name: Name,
    pub env: Arc<Mutex<HashMap<Id, ServiceInfo>>>,
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

    fn get_handler(&self) -> Handler {
        Self::default_handler()
    }
}

impl Actuator for TemperatureActuator {}

impl TemperatureActuator {
    pub fn new(id: Id, name: Name) -> TemperatureActuator {
        TemperatureActuator {
            id,
            name,
            env: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn start(ip: IpAddr, port: u16, id: Id, name: Name) {
        let device = Self::new(id, name);

        let mut targets = HashMap::new();
        targets.insert("_controller".into(), &device.env);

        device.run(ip, port, "_actuator", targets);
    }
}
