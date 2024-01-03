use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use mdns_sd::ServiceInfo;

use datum::kind::Kind;
use datum::unit::Unit;
use datum::Datum;
use device::address::Address;
use device::id::Id;
use device::message::Message;
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
    data: Arc<Mutex<VecDeque<Datum>>>,
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

    /// By default, a `Sensor` responds to any request with the latest `Datum`.
    fn get_handler(&self) -> Handler {
        let self_name = self.get_name().clone();

        Box::new(move |stream| {
            if let Ok(message) = Message::read(stream) {
                let body = format!("[Device] ignoring message: {}", message);
                let response = Message::respond_not_implemented().with_body(body);
                response.write(stream)
            } else {
                Self::handler_failure(
                    self_name.clone(),
                    stream,
                    "unable to read Message from stream",
                )
            }
        })
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
            data: Arc::new(Mutex::new(VecDeque::new())),
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

    fn get_data(&self) -> &Arc<Mutex<VecDeque<Datum>>> {
        &self.data
    }
}
