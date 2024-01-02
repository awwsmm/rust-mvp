use std::sync::{Arc, Mutex};

use mdns_sd::ServiceInfo;

use actuator::Actuator;
use device::address::Address;
use device::id::Id;
use device::message::Message;
use device::model::Model;
use device::name::Name;
use device::{Device, Handler};

pub mod command;

pub struct TemperatureActuator {
    id: Id,
    name: Name,
    environment: Arc<Mutex<Option<ServiceInfo>>>,
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

    /// By default, an `Actuator` forwards all incoming requests to the `Environment`.
    fn get_handler(&self) -> Handler {
        Box::new(move |stream| {
            let response = Message::respond_not_implemented();
            response.write(stream)
        })
    }
}

impl Actuator for TemperatureActuator {
    fn new(id: Id, name: Name, address: Address) -> Self {
        Self {
            id,
            name,
            environment: Arc::new(Mutex::new(None)),
            address,
        }
    }

    fn get_environment(&self) -> &Arc<Mutex<Option<ServiceInfo>>> {
        &self.environment
    }

    fn get_environment_info(&self) -> Option<ServiceInfo> {
        let lock = self.environment.lock();
        let guard = lock.unwrap();
        guard.clone()
    }
}
