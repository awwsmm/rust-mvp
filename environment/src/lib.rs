use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use rand::{thread_rng, Rng};

use datum::{Datum, DatumUnit, DatumValueType};
use device::id::Id;
use device::model::Model;
use device::name::Name;
use device::{Device, Handler};

use crate::generator::DatumGenerator;

mod generator;

/// A test-only example environment which produces data detected by `Sensor`s.
///
/// The `Environment` can be mutated by `Actuator`s.
pub struct Environment {
    name: Name,
    id: Id,
    #[allow(dead_code)] // remove this ASAP
    attributes: Arc<Mutex<HashMap<Id, DatumGenerator>>>,
    address: String,
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

    fn get_group() -> String {
        String::from("_environment")
    }

    fn get_address(&self) -> &String {
        &self.address
    }

    // TODO Environment should respond to HTTP requests from Actuators and Sensors.
    fn get_handler(&self) -> Handler {
        let sender_name = self.get_name().to_string().clone();
        let sender_address = self.get_address().clone();

        let attributes = Arc::clone(&self.attributes);

        Box::new(move |stream| {
            if let Ok(message) =
                Self::ack_and_parse_request(sender_name.clone(), sender_address.clone(), stream)
            {
                println!(
                    "[Environment] received message (ignoring for now)\nvvvvvvvvvv\n{}\n^^^^^^^^^^",
                    message
                );

                match (
                    message.headers.get("id"),
                    message.headers.get("kind"),
                    message.headers.get("unit"),
                ) {
                    (Some(id), Some(kind), Some(unit)) => {
                        match (
                            Id::new(id),
                            DatumValueType::parse(kind),
                            DatumUnit::parse(unit),
                        ) {
                            (id, Ok(kind), Ok(unit)) => {
                                let datum = Self::get(attributes.clone(), &id, kind, unit);

                                println!("[Environment] generated Datum: {}", datum)
                            }
                            _ => println!("[Environment] cannot parse id, kind, or unit"),
                        }
                    }
                    _ => println!("[Environment] cannot parse headers to get appropriate data"),
                }

                // let contents = Self::get_datum().to_string();
                // let message = Message::respond_ok_with_body(HashMap::new(), contents.as_str());

                // stream.write_all(message.to_string().as_bytes()).unwrap();
            }
        })
    }

    fn start(ip: IpAddr, port: u16, id: Id, name: Name) -> JoinHandle<()> {
        let host = ip.clone().to_string();
        let address = <Self as Device>::address(host, port.to_string());

        std::thread::spawn(move || {
            let device = Self::new(id, name, address);
            device.run(ip, port, "_environment", HashMap::new());
        })
    }
}

impl Environment {
    pub fn new(id: Id, name: Name, address: String) -> Self {
        Self {
            name,
            id,
            attributes: Arc::new(Mutex::new(HashMap::new())),
            address,
        }
    }

    pub fn start_default(ip: IpAddr, port: u16) -> JoinHandle<()> {
        Self::start(ip, port, Id::new("environment"), Name::new("Environment"))
    }

    #[allow(dead_code)] // remove this ASAP
    fn set(&self, id: Id, generator: DatumGenerator) {
        let mut attributes = self.attributes.lock().unwrap();
        attributes.insert(id, generator);
    }

    #[allow(dead_code)] // remove this ASAP
    fn get(
        attributes: Arc<Mutex<HashMap<Id, DatumGenerator>>>,
        id: &Id,
        kind: DatumValueType,
        unit: DatumUnit,
    ) -> Datum {
        let mut attributes = attributes.lock().unwrap();
        match attributes.get_mut(id) {
            Some(generator) => generator.generate(),
            None => {
                // we need to return the type (bool, f32, i32) of data the Sensor expects
                let mut rng = thread_rng();
                let generator = match kind {
                    DatumValueType::Bool => {
                        let initial = false; // first value returned
                        generator::bool_alternating(initial, unit)
                    }
                    DatumValueType::Int => {
                        let slope = rng.gen_range(-10..10); // arbitrarily selected range of slopes
                        let noise = rng.gen_range(0..2); // arbitrary selected range of noise values
                        generator::time_dependent::i32_linear(slope, noise, unit)
                    }
                    DatumValueType::Float => {
                        let slope = rng.gen_range(-0.10..0.10); // arbitrarily selected range of slopes
                        let noise = rng.gen_range(0.0..0.10); // arbitrary selected range of noise values
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
}
