use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use mdns_sd::ServiceInfo;

use datum::kind::Kind;
use datum::unit::Unit;
use device::address::Address;
use device::id::Id;
use device::model::Model;
use device::name::Name;
use device::{Device, Handler, Targets};
use sensor::Sensor;

pub struct TemperatureSensor {
    id: Id,
    name: Name,
    environment: Targets,
    controller: Targets,
    address: Address,
}

impl Device for TemperatureSensor {
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
}

impl Sensor for TemperatureSensor {
    fn get_environment(&self) -> Option<ServiceInfo> {
        let lock = self.environment.lock();
        let guard = lock.unwrap();
        guard.get(&Id::new("environment")).cloned()
    }

    fn get_controller(&self) -> Option<ServiceInfo> {
        let lock = self.controller.lock();
        let guard = lock.unwrap();
        guard.get(&Id::new("controller")).cloned()
    }

    fn get_datum_value_type(&self) -> Kind {
        Kind::Float
    }

    fn get_datum_unit(&self) -> Unit {
        Unit::DegreesC
    }

    fn targets_by_group(&self) -> HashMap<String, Targets> {
        let mut map = HashMap::new();
        map.insert("_controller".into(), Arc::clone(&self.controller));
        map.insert("_environment".into(), Arc::clone(&self.environment));
        map
    }

    fn new(id: Id, name: Name, address: Address) -> TemperatureSensor {
        TemperatureSensor {
            id,
            name,
            environment: Arc::new(Mutex::new(HashMap::new())),
            controller: Arc::new(Mutex::new(HashMap::new())),
            address,
        }
    }
}
