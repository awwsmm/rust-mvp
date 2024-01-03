use std::collections::HashMap;
use std::net::{IpAddr, TcpStream};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;

use mdns_sd::ServiceDaemon;
use rand::{thread_rng, Rng};

use datum::kind::Kind;
use datum::unit::Unit;
use datum::Datum;
use device::address::Address;
use device::id::Id;
use device::message::Message;
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

    // TODO Environment should respond to HTTP requests from Actuators.
    fn get_handler(&self) -> Handler {
        // Anything which depends on self must be cloned outside of the |stream| lambda.
        // We cannot refer to `self` inside of this lambda.

        let self_name = self.name.clone();
        let attributes = Arc::clone(&self.attributes);

        Box::new(move |stream| {
            if let Ok(message) = Message::read(stream) {
                if message.start_line.starts_with("GET /datum/") {
                    // Ask the Environment for the latest Datum for a Sensor by its ID.
                    //
                    // There are two possibilities here:
                    //   1. the Environment knows about this Sensor (its ID) already
                    //   2. the Environment doesn't know about this Sensor
                    //
                    // In case (1), all we need is the ID. In case (2), we also need to know the kind of data to generate.

                    let id = message
                        .start_line
                        .trim_start_matches("GET /datum/")
                        .trim_end_matches(" HTTP/1.1");
                    let id = Id::new(id);

                    let mut attributes = attributes.lock().unwrap();

                    fn success(stream: &mut TcpStream, datum: Datum) {
                        let datum = datum.to_string();
                        println!(
                            "[Environment] generated Datum to send back to sensor: {}",
                            datum
                        );
                        let response = Message::respond_ok().with_body(datum);
                        response.write(stream)
                    }

                    match attributes.get_mut(&id) {
                        None => {
                            // if this Sensor ID is unknown, we can still generate data for it if the user has included the 'kind' and 'unit' headers
                            //     ex: curl --header "kind: bool" --header "unit: °C" 10.12.50.26:5454/datum/my_id
                            match (message.headers.get("kind"), message.headers.get("unit")) {
                                (Some(kind), Some(unit)) => {
                                    match (Kind::parse(kind), Unit::parse(unit)) {
                                        (Ok(kind), Ok(unit)) => {
                                            let datum = Self::register_new(
                                                &mut attributes,
                                                &id,
                                                kind,
                                                unit,
                                            )
                                            .to_string();
                                            println!("[Environment] generated Datum to send back to sensor: {}", datum);
                                            let response = Message::respond_ok().with_body(datum);
                                            response.write(stream)
                                        }
                                        _ => {
                                            let msg = "could not parse required headers";
                                            Self::handler_failure(self_name.clone(), stream, msg)
                                        }
                                    }
                                }
                                _ => {
                                    let msg = format!("unknown Sensor ID '{}'. To register a new sensor, you must include 'kind' and 'unit' headers in your request", id);
                                    Self::handler_failure(self_name.clone(), stream, msg.as_str())
                                }
                            }
                        }
                        Some(generator) => {
                            // if this Sensor ID is known, we can generate data for it without any additional information
                            //     ex: curl 10.12.50.26:5454/datum/my_id
                            success(stream, generator.generate())
                        }
                    }
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

impl Environment {
    fn new(id: Id, name: Name, address: Address) -> Self {
        Self {
            name,
            id,
            attributes: Arc::new(Mutex::new(HashMap::new())),
            address,
        }
    }

    fn register_new(
        attributes: &mut MutexGuard<HashMap<Id, DatumGenerator>>,
        id: &Id,
        kind: Kind,
        unit: Unit,
    ) -> Datum {
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
