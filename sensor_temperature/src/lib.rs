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

        // Anything which depends on self must be cloned outside of the |stream| lambda.
        // We cannot refer to `self` inside of this lambda.
        let self_data = Arc::clone(self.get_data());

        Box::new(move |stream| {
            if let Ok(message) = Message::read(stream) {
                if message.start_line == "GET /data HTTP/1.1" {
                    // get all of the data in this Sensor's buffer
                    //     ex: curl 10.12.50.26:5454/data

                    let data = self_data.lock().unwrap();
                    let data: Vec<String> = data.iter().map(|d| d.to_string()).collect();
                    let data = data.join("\r\n");

                    let response = Message::respond_ok().with_body(data);
                    response.write(stream)
                } else {
                    // TODO implement other endpoints
                    let msg = format!("cannot parse request: {}", message.start_line);
                    Self::handler_failure(self_name.clone(), stream, msg.as_str())
                }
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
