use std::collections::HashMap;
use std::io::Write;
use std::net::IpAddr;
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread::JoinHandle;

use mdns_sd::ServiceDaemon;

use actuator_temperature::command::Command;
use datum::kind::Kind;
use datum::unit::Unit;
use datum::Datum;
use device::id::Id;
use device::message::Message;
use device::model::Model;
use device::name::Name;
use device::{Device, Handler};

use crate::generator::{Coefficients, DatumGenerator};

mod generator;

/// A test-only example environment which produces data detected by `Sensor`s.
///
/// The `Environment` can be mutated by `Actuator`s.
pub struct Environment {
    name: Name,
    id: Id,
    attributes: Arc<Mutex<HashMap<Id, DatumGenerator>>>,
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

    // coverage: off
    // routing can be verified by inspection
    fn get_handler(&self) -> Handler {
        // Anything which depends on self must be cloned outside of the |stream| lambda.
        // We cannot refer to `self` inside of this lambda.
        let self_name = self.name.clone();
        let self_attributes = Arc::clone(&self.attributes);

        Box::new(move |stream| {
            if let Ok(message) = Message::read(stream) {
                if message.start_line.starts_with("GET /datum/") {
                    Self::handle_get_datum(stream, message, &self_name, &self_attributes)
                } else if message.start_line == "POST /command HTTP/1.1" {
                    Self::handle_post_command(stream, message, &self_name, &self_attributes)
                } else {
                    let msg = format!("cannot parse request: {}", message.start_line);
                    Self::handler_failure(self_name.clone(), stream, msg.as_str())
                }
            } else {
                Self::handler_failure(self_name.clone(), stream, "unable to read Message from stream")
            }
        })
    }
    // coverage: on
}

impl Environment {
    fn new(id: Id, name: Name) -> Self {
        Self {
            name,
            id,
            attributes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn register_new(attributes: &mut MutexGuard<HashMap<Id, DatumGenerator>>, id: &Id, kind: Kind, unit: Unit) -> Datum {
        match attributes.get_mut(id) {
            Some(generator) => generator.generate(),
            None => {
                // we need to return the type (bool, f32, i32) of data the Sensor expects
                let generator = match kind {
                    Kind::Bool => {
                        unimplemented!()
                    }
                    Kind::Int => {
                        unimplemented!()
                    }
                    Kind::Float => {
                        let coefficients = Coefficients::new(0.0, 0.0, 5.0, 10000.0, 0.0);
                        let noise = 0.5;
                        DatumGenerator::new(coefficients, noise, unit)
                    }
                };

                // register this Datum generator to this Id
                attributes.insert(id.clone(), generator);

                // generate a random value
                attributes.get_mut(id).unwrap().generate()
            }
        }
    }

    fn handle_get_datum(stream: &mut impl Write, message: Message, self_name: &Name, attributes: &Arc<Mutex<HashMap<Id, DatumGenerator>>>) {
        // Ask the Environment for the latest Datum for a Sensor by its ID.
        //
        // There are two possibilities here:
        //   1. the Environment knows about this Sensor (its ID) already
        //   2. the Environment doesn't know about this Sensor
        //
        // In case (1), all we need is the ID. In case (2), we also need to know the kind of data to generate.

        let id = message.start_line.trim_start_matches("GET /datum/").trim_end_matches(" HTTP/1.1");
        let id = Id::new(id);

        let mut attributes = attributes.lock().unwrap();

        fn success(stream: &mut impl Write, datum: Datum) {
            let datum = datum.to_string();
            println!("[Environment] generated Datum to send back to sensor: {}", datum);
            let response = Message::respond_ok().with_body(datum);
            response.write(stream)
        }

        match attributes.get_mut(&id) {
            None => {
                // if this Sensor ID is unknown, we can still generate data for it if the user has included the 'kind' and 'unit' headers
                //     ex: curl --header "kind: bool" --header "unit: °C" 10.12.50.26:5454/datum/my_id
                match (message.header("kind"), message.header("unit")) {
                    (Some(kind), Some(unit)) => match (Kind::parse(kind), Unit::parse(unit)) {
                        (Ok(kind), Ok(unit)) => {
                            let datum = Self::register_new(&mut attributes, &id, kind, unit);
                            success(stream, datum);
                        }
                        _ => {
                            let msg = "could not parse required headers";
                            Self::handler_failure(self_name.clone(), stream, msg)
                        }
                    },
                    _ => {
                        let msg = format!(
                            "unknown Sensor ID '{}'. To register a new sensor, you must include 'kind' and 'unit' headers in your request",
                            id
                        );
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
    }

    fn handle_post_command(stream: &mut impl Write, message: Message, self_name: &Name, attributes: &Arc<Mutex<HashMap<Id, DatumGenerator>>>) {
        fn success(stream: &mut impl Write) {
            println!("[Environment] updated generator for Sensor");
            let response = Message::respond_ok();
            response.write(stream)
        }

        // Tell the Environment to update its State via a Command.
        //     ex: curl 10.12.50.26:5454/command -d '{"name":"HeatBy","value":"25"}' --header "id: my_id" --header "model: thermo5000"
        match (message.header("id"), message.header("model")) {
            (Some(id), Some(model)) => match (Id::new(id), Model::parse(model)) {
                (id, Ok(model)) => match model {
                    Model::Controller => {
                        let msg = "does not accept Commands directly from the Controller";
                        Self::handler_failure(self_name.clone(), stream, msg)
                    }
                    Model::Environment => {
                        let msg = "does not accept Commands from itself";
                        Self::handler_failure(self_name.clone(), stream, msg)
                    }
                    Model::Unsupported => {
                        let msg = "unsupported device";
                        Self::handler_failure(self_name.clone(), stream, msg)
                    }
                    Model::Thermo5000 => match message.body.as_ref().map(Command::parse) {
                        Some(Ok(command)) => {
                            println!("[Environment] successfully parsed command: {}", command);

                            let mut attributes = attributes.lock().unwrap();

                            match attributes.contains_key(&id) {
                                false => {
                                    let msg = format!("cannot update generator for unknown id: {}", id);
                                    Self::handler_failure(self_name.clone(), stream, msg.as_str())
                                }
                                true => {
                                    let generator = attributes.get_mut(&id).unwrap();
                                    match command {
                                        Command::CoolBy(delta) => {
                                            generator.coefficients.constant -= delta * 0.01;
                                        }
                                        Command::HeatBy(delta) => {
                                            generator.coefficients.constant += delta * 0.01;
                                        }
                                    }

                                    success(stream)
                                }
                            }
                        }
                        _ => {
                            let msg = format!("could not parse \"{:?}\" as Thermo5000 Command", message.body);
                            Self::handler_failure(self_name.clone(), stream, msg.as_str())
                        }
                    },
                },
                _ => {
                    let msg = "could not parse required headers";
                    Self::handler_failure(self_name.clone(), stream, msg)
                }
            },
            _ => {
                let msg = "missing required headers. 'id' and 'model' headers are required to update a generator.";
                Self::handler_failure(self_name.clone(), stream, msg)
            }
        }
    }

    // coverage: off
    // this is very difficult to test outside of an integration test
    pub fn start(ip: IpAddr, port: u16, id: Id, name: Name, group: String) -> JoinHandle<()> {
        std::thread::spawn(move || {
            let device = Self::new(id, name);

            let mdns = ServiceDaemon::new().unwrap();

            device.respond(ip, port, group.as_str(), mdns)
        })
    }
    // coverage: on
}

#[cfg(test)]
mod environment_tests {
    use super::*;

    #[test]
    fn test_get_name() {
        let expected = Name::new("myName");
        let controller = Environment::new(Id::new("myId"), expected.clone());
        let actual = controller.get_name();
        let expected = &expected;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_get_id() {
        let expected = Id::new("myId");
        let controller = Environment::new(expected.clone(), Name::new("myName"));
        let actual = controller.get_id();
        let expected = &expected;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_get_model() {
        let actual = Environment::get_model();
        let expected = Model::Environment;
        assert_eq!(actual, expected);
    }
}
