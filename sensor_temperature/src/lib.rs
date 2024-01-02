use std::sync::{Arc, Mutex};

use mdns_sd::ServiceInfo;

use datum::kind::Kind;
use datum::unit::Unit;
use device::{Device, Handler};
use device::address::Address;
use device::id::Id;
use device::message::Message;
use device::model::Model;
use device::name::Name;
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

    /// By default, a `Sensor` responds to any request with the latest `Datum`.
    fn get_handler(&self) -> Handler {
        Box::new(move |stream| {
            if let Ok(message) = Message::read(stream) {
                let body = format!("[Device] ignoring message: {}", message);
                let response = Message::respond_not_implemented().with_body(body);
                response.write(stream)

            } else {
                let body = "unable to read Message from TcpStream";
                println!("[Device] {}", body);
                let response = Message::respond_bad_request().with_body(body);
                response.write(stream)
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
