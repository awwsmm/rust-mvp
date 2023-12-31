use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use mdns_sd::ServiceInfo;

use actuator::Actuator;
use device::address::Address;
use device::id::Id;
use device::model::Model;
use device::name::Name;
use device::{Device, Handler, Targets};

pub mod command;

pub struct TemperatureActuator {
    id: Id,
    name: Name,
    pub env: Targets,
    address: Address,
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

    fn get_address(&self) -> Address {
        self.address
    }

    fn get_handler(&self) -> Handler {
        self.default_handler()
    }

    fn targets_by_group(&self) -> HashMap<String, Targets> {
        let mut map = HashMap::new();
        map.insert("_environment".into(), Arc::clone(&self.env));
        map
    }

    fn new(id: Id, name: Name, address: Address) -> Self {
        Self {
            id,
            name,
            env: Arc::new(Mutex::new(HashMap::new())),
            address,
        }
    }
}

impl Actuator for TemperatureActuator {
    fn get_environment(&self) -> Option<ServiceInfo> {
        let lock = self.env.lock();
        let guard = lock.unwrap();
        guard.get(&Id::new("environment")).cloned()
    }
}
