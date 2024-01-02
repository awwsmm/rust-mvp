use std::sync::{Arc, Mutex};

use mdns_sd::ServiceInfo;

use datum::kind::Kind;
use datum::unit::Unit;
use device::address::Address;
use device::id::Id;
use device::model::Model;
use device::name::Name;
use device::{Device, Handler};
use sensor::Sensor;

pub struct TemperatureSensor {
    id: Id,
    name: Name,
    environment: Arc<Mutex<Option<ServiceInfo>>>,
    controller: Arc<Mutex<Option<ServiceInfo>>>,
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
    fn new(id: Id, name: Name, address: Address) -> TemperatureSensor {
        TemperatureSensor {
            id,
            name,
            environment: Arc::new(Mutex::new(None)),
            controller: Arc::new(Mutex::new(None)),
            address,
        }
    }

    fn get_environment(&self) -> &Arc<Mutex<Option<ServiceInfo>>> {
        &self.environment
    }

    fn get_controller(&self) -> &Arc<Mutex<Option<ServiceInfo>>> {
        &self.controller
    }

    fn get_environment_info(&self) -> Option<ServiceInfo> {
        let lock = self.environment.lock();
        let guard = lock.unwrap();
        guard.clone()
    }

    fn get_controller_info(&self) -> Option<ServiceInfo> {
        let lock = self.controller.lock();
        let guard = lock.unwrap();
        guard.clone()
    }

    fn get_datum_value_type(&self) -> Kind {
        Kind::Float
    }

    fn get_datum_unit(&self) -> Unit {
        Unit::DegreesC
    }
}
