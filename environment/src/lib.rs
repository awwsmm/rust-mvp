use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use mdns_sd::ServiceDaemon;
use rand::{Rng, thread_rng};

use datum::Datum;
use datum::kind::Kind;
use datum::unit::Unit;
use device::{Device, Handler};
use device::address::Address;
use device::id::Id;
use device::message::Message;
use device::model::Model;
use device::name::Name;

use crate::generator::DatumGenerator;

mod generator;

/// A test-only example environment which produces data detected by `Sensor`s.
///
/// The `Environment` can be mutated by `Actuator`s.
pub struct Environment {
    name: Name,
    id: Id,
    attributes: Arc<Mutex<HashMap<Id, DatumGenerator>>>,
    address: Address,
}

impl Device for Environment {
    fn get_name(&self) -> &Name {
        &self.name
    }

    fn get_id(&self) -> &Id {
        &self.id
    }

    fn get_model() -> Model {
        Model::Environment
    }

    fn get_address(&self) -> Address {
        self.address
    }

    // TODO Environment should respond to HTTP requests from Actuators and Sensors.
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

impl Environment {
    fn new(id: Id, name: Name, address: Address) -> Self {
        Self {
            name,
            id,
            attributes: Arc::new(Mutex::new(HashMap::new())),
            address,
        }
    }

    fn get(
        attributes: Arc<Mutex<HashMap<Id, DatumGenerator>>>,
        id: &Id,
        kind: Kind,
        unit: Unit,
    ) -> Datum {
        let mut attributes = attributes.lock().unwrap();
        match attributes.get_mut(id) {
            Some(generator) => generator.generate(),
            None => {
                // we need to return the type (bool, f32, i32) of data the Sensor expects
                let mut rng = thread_rng();
                let generator = match kind {
                    Kind::Bool => {
                        let initial = false; // first value returned
                        generator::bool_alternating(initial, unit)
                    }
                    Kind::Int => {
                        let slope = rng.gen_range(-10..10); // arbitrarily selected range of slopes
                        let noise = rng.gen_range(0..2); // arbitrary selected range of noise values
                        generator::time_dependent::i32_linear(slope, noise, unit)
                    }
                    Kind::Float => {
                        let slope = rng.gen_range(-0.001..0.001); // arbitrarily selected range of slopes
                        let noise = rng.gen_range(0.0..0.00001); // arbitrary selected range of noise values
                        generator::time_dependent::f32_linear(slope, noise, unit)
                    }
                };

                // register this Datum generator to this Id
                attributes.insert(id.clone(), generator);

                // generate a random value
                attributes.get_mut(id).unwrap().generate()
            }
        }
    }

    pub fn start(ip: IpAddr, port: u16, id: Id, name: Name, group: String) -> JoinHandle<()> {
        std::thread::spawn(move || {
            let device = Self::new(id, name, Address::new(ip, port));

            let mdns = ServiceDaemon::new().unwrap();

            device.respond(ip, port, group.as_str(), mdns)
        })
    }
}
