use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use mdns_sd::ServiceDaemon;
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
    attributes: Mutex<HashMap<Id, DatumGenerator>>,
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
        Box::new(|stream, _mdns| {
            if let Ok(message) = Self::parse_http_request(stream) {
                println!(
                    "[Environment] received\n----------\n{}\n----------",
                    message
                );

                // let contents = Self::get_datum().to_string();
                // let message = Message::respond_ok_with_body(HashMap::new(), contents.as_str());

                // stream.write_all(message.to_string().as_bytes()).unwrap();
            }
        })
    }

    fn start(
        ip: IpAddr,
        port: u16,
        id: Id,
        name: Name,
        mdns: Arc<ServiceDaemon>,
    ) -> JoinHandle<()> {
        let host = ip.clone().to_string();
        let address = <Self as Device>::address(host, port.to_string());

        std::thread::spawn(move || {
            println!(">>> [environment start] SPAWNED A NEW THREAD");
            let device = Self::new(id, name, address);
            device.run(ip, port, "_environment", HashMap::new(), mdns);
        })
    }
}

impl Environment {
    pub fn new(id: Id, name: Name, address: String) -> Self {
        Self {
            name,
            id,
            attributes: Mutex::new(HashMap::new()),
            address,
        }
    }

    pub fn start_default(ip: IpAddr, port: u16, mdns: Arc<ServiceDaemon>) -> JoinHandle<()> {
        Self::start(
            ip,
            port,
            Id::new("environment"),
            Name::new("environment"),
            mdns,
        )
    }

    #[allow(dead_code)] // remove this ASAP
    fn set(&self, id: Id, generator: DatumGenerator) {
        let mut attributes = self.attributes.lock().unwrap();
        attributes.insert(id, generator);
    }

    #[allow(dead_code)] // remove this ASAP
    fn get(&mut self, id: &Id, kind: DatumValueType, unit: DatumUnit) -> Datum {
        let mut attributes = self.attributes.lock().unwrap();
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

    // #[allow(dead_code)] // remove this ASAP
    // pub fn handle_request(&mut self, message: &str) -> String {
    //     if message.starts_with("POST /set/") {
    //         // if the Environment gets a command from an actuator with a Device::Id that it is not
    //         // yet aware of, it should ignore it
    //
    //         // Not complete, but this is the general idea
    //         // Extract ID and command
    //         let (id, command) = self.extract_command(message);
    //         match self.execute_command(&id, &command) {
    //             Some(datum) => format!("HTTP/1.1 200 OK\r\n\r\n{:?}", datum),
    //             None => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
    //         }
    //     } else if message.starts_with("GET /get/") {
    //         // if the Environment gets a message from a sensor with a Device::Id that it is not
    //         // yet aware of, it should save the Id and pick from a random data generator
    //
    //         let parsed = Environment::parse_get_request(message);
    //
    //         if let Ok((id, value_type, unit)) = parsed {
    //             let datum = self.get(&id, value_type, unit);
    //             format!("HTTP/1.1 200 OK\r\n\r\n{:?}", datum)
    //         } else {
    //             let msg = parsed.unwrap_err();
    //             format!("HTTP/1.1 400 Bad Request\r\n\r\n{}", msg)
    //         }
    //     } else {
    //         "HTTP/1.1 404 Not Found\r\n\r\n".to_string()
    //     }
    // }

    // #[allow(dead_code)] // remove this ASAP
    // fn parse_get_request(message: &str) -> Result<(Id, DatumValueType, DatumUnit), String> {
    //     // example message: "GET /get/test_id/float/°C"
    //     let mut parts = message.split('/');
    //
    //     parts.next(); // throw out "GET"
    //     parts.next(); // throw out "get"
    //
    //     match (parts.next(), parts.next(), parts.next()) {
    //         (Some(id), Some(value_type), Some(unit)) => {
    //             match (DatumValueType::parse(value_type), DatumUnit::parse(unit)) {
    //                 (Ok(value_type), Ok(unit)) => Ok((Id::new(id), value_type, unit)),
    //                 (Ok(_), Err(msg)) => Err(msg),
    //                 (Err(msg), Ok(_)) => Err(msg),
    //                 (Err(msg1), Err(msg2)) => Err(format!("{}\n{}", msg1, msg2)),
    //             }
    //         }
    //         _ => Err(format!(
    //             "Cannot split {} into /get/<sensor_id>/<type>/<unit>",
    //             message
    //         )),
    //     }
    // }
}

#[cfg(test)]
mod env_tests {
    use chrono::{DateTime, Utc};

    use datum::{DatumUnit, DatumValue};

    use super::*;

    #[test]
    fn test_set_and_get_datum() {
        let mut environment = Environment::new(Id::new(""), Name::new(""), "".into());

        let id = Id::new("test_id");
        let value_type = DatumValueType::Int;
        let unit = DatumUnit::Unitless;

        let constant = |_: DateTime<Utc>| -> DatumValue { DatumValue::Int(42) };

        let generator = DatumGenerator::new(Box::new(constant), unit);

        environment.set(id.clone(), generator);
        let datum = environment.get(&id, value_type, unit);

        assert_eq!(datum.value, DatumValue::Int(42));
        assert_eq!(datum.unit, unit);
    }

    #[test]
    fn test_get_with_existing_generator() {
        let mut env = Environment::new(Id::new(""), Name::new(""), "".into());
        let id = Id::new("test_id");
        let unit = DatumUnit::DegreesC;

        // Create a generator for this Id
        let f = |_| -> DatumValue { DatumValue::Int(42) };
        let generator = DatumGenerator::new(Box::new(f), unit);
        env.attributes.lock().unwrap().insert(id.clone(), generator);

        // Test get method with existing generator
        let datum = env.get(&id, DatumValueType::Int, unit);
        assert_eq!(datum.value, DatumValue::Int(42));
        assert_eq!(datum.unit, unit);
    }

    #[test]
    fn test_get_with_new_bool_generator() {
        let mut env = Environment::new(Id::new(""), Name::new(""), "".into());
        let id = Id::new("new_bool_id");
        let unit = DatumUnit::Unitless;

        // Test get method with new generator for bool type
        let datum = env.get(&id, DatumValueType::Bool, unit);
        match datum.value {
            DatumValue::Bool(_) => (),
            _ => panic!("Expected Bool, found {:?}", datum.value),
        }
        assert_eq!(datum.unit, unit);
    }

    #[test]
    fn test_get_with_new_int_generator() {
        let mut env = Environment::new(Id::new(""), Name::new(""), "".into());
        let id = Id::new("new_int_id");
        let unit = DatumUnit::PoweredOn;

        // Test get method with new generator for Int type
        let datum = env.get(&id, DatumValueType::Int, unit);
        match datum.value {
            DatumValue::Int(_) => (),
            _ => panic!("Expected Int, found {:?}", datum.value),
        }
        assert_eq!(datum.unit, unit);
    }

    #[test]
    fn test_get_with_new_float_generator() {
        let mut env = Environment::new(Id::new(""), Name::new(""), "".into());
        let id = Id::new("new_float_id");
        let unit = DatumUnit::DegreesC;

        // Test get method with new generator for float type
        let datum = env.get(&id, DatumValueType::Float, unit);
        match datum.value {
            DatumValue::Float(_) => (),
            _ => panic!("Expected Float, found {:?}", datum.value),
        }
        assert_eq!(datum.unit, unit);
    }
}
