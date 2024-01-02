use std::collections::HashMap;
use std::net::{IpAddr, TcpStream};
use std::sync::Arc;
use std::thread::JoinHandle;

use mdns_sd::ServiceDaemon;

use datum::Datum;
use device::{Device, Handler, Targets};
use device::address::Address;
use device::id::Id;
use device::message::Message;
use device::model::Model;
use device::name::Name;

use crate::assessor::DEFAULT_ASSESSOR;
use crate::state::State;

mod assessor;
mod state;

/// The Controller queries the `Sensor`s for `Datum`s and sends commands to the `Actuator`s.
///
/// The Controller logically ties a `Sensor` to its corresponding `Actuator`. It queries the
/// `Sensor` for its data, and makes a decision based on its state and the `Sensor` data, then
/// constructs an appropriate command to send to that `Sensor`'s `Actuator`.
///
/// The `Controller`'s state can be queried by an HTML frontend, so some historic data is held
/// in memory.
pub struct Controller {
    name: Name,
    id: Id,
    state: State,
    address: Address,
}

impl Device for Controller {
    fn get_name(&self) -> &Name {
        &self.name
    }

    fn get_id(&self) -> &Id {
        &self.id
    }

    fn get_model() -> Model {
        Model::Controller
    }

    fn get_address(&self) -> Address {
        self.address
    }

    // TODO Controller should respond to HTTP requests from Sensors.
    fn get_handler(&self) -> Handler {
        let sender_name = self.get_name().to_string().clone();
        let sender_address = self.get_address();

        let assessors = Arc::clone(&self.state.assessors);
        let assessors = assessors.lock();
        let assessors = assessors.unwrap().clone();

        let actuators = Arc::clone(&self.state.actuators);
        let actuators = actuators.lock();
        let actuators = actuators.unwrap().clone();

        Box::new(move |stream| {
            if let Ok(request) =
                Self::ack_and_parse_request(sender_name.as_str(), sender_address, stream)
            {
                println!(
                    "[Controller] received message (ignoring for now)\nvvvvvvvvvv\n{}\n^^^^^^^^^^",
                    request
                );

                if request.headers.get("sender_name") == Some(&String::from("Web App")) {
                    println!(
                        "[Controller] received request from Web App\nvvvvvvvvvv\n{}\n^^^^^^^^^^",
                        request
                    );
                } else {
                    println!("[Controller] received request from (what is assumed to be a) Sensor\nvvvvvvvvvv\n{}\n^^^^^^^^^^", request);

                    println!(
                        "[Controller] available assessors: {:?}",
                        assessors.keys().map(|each| each.to_string())
                    );

                    let id = Id::new(request.headers.get("id").unwrap());
                    let model = Model::parse(request.headers.get("model").unwrap()).unwrap();

                    if let Some(assessor) = assessors
                        .get(&id)
                        .or_else(|| DEFAULT_ASSESSOR.get(model.to_string().as_str()))
                    {
                        println!("[Controller] found assessor");

                        let datum = Datum::parse(request.body.unwrap().as_str()).unwrap();

                        println!("[Controller] parsed Datum from request body: {}", datum);

                        match (assessor.assess)(&datum) {
                            None => println!("[Controller] assessed Datum, but will not produce Command for Actuator"),
                            Some(command) => {
                                println!(
                                    "[Controller] sending command to Actuator: {}",
                                    command
                                );

                                match actuators.get(&id) {
                                    None => println!("[Controller] cannot find Actuator with id: {}", id),
                                    Some(actuator) => {
                                        let actuator = <Self as Device>::extract_address(actuator).to_string();
                                        println!("[Sensor] connecting to Actuator @ {}", actuator);

                                        let mut stream = TcpStream::connect(actuator).unwrap();

                                        let command = Message::ping(
                                            sender_name.as_str(),
                                            sender_address
                                        ).with_body(
                                            command.to_string()
                                        );

                                        println!("[Controller] sending Command to Actuator\nvvvvvvvvvv\n{}\n^^^^^^^^^^", command);

                                        command.write(&mut stream);

                                    }
                                }

                            }
                        }
                    } else {
                        println!(
                            "[Controller] assessor does not contain id: {}\nknown ids: {:?}",
                            id,
                            assessors.keys()
                        )
                    }
                }
            }
        })
    }
}

impl Controller {
    fn new(id: Id, name: Name, address: Address) -> Self {
        Self {
            name,
            id,
            state: State::new(),
            address,
        }
    }

    #[allow(dead_code)] // FIXME remove ASAP
    fn is_supported(model: &Model) -> bool {
        DEFAULT_ASSESSOR.contains_key(model.to_string().as_str())
    }

    /// Pings the latest `Sensor` so that it can (asynchronously) send a response containing the latest `Datum`.
    pub fn ping_sensor(sender_name: &str, return_address: Address, sensor_address: Address) {
        let mut tcp_stream = TcpStream::connect(sensor_address.to_string()).unwrap();

        // send the minimum possible payload. We only want to ping the Sensor
        // see: https://stackoverflow.com/a/9734866
        let ping = Message::ping(sender_name, return_address);
        ping.write(&mut tcp_stream);
    }

    /// Returns all of the `Sensor`s of which the `Controller` is aware.
    pub fn get_sensors(&self) -> &Targets {
        &self.state.sensors
    }

    pub fn start(ip: IpAddr, port: u16, id: Id, name: Name, group: String) -> JoinHandle<()> {
        std::thread::spawn(move || {
            let device = Self::new(id, name, Address::new(ip, port));

            let mut targets = HashMap::new();
            targets.insert("_sensor", Arc::clone(&device.state.sensors));
            targets.insert("_actuator", Arc::clone(&device.state.actuators));

            let mdns = ServiceDaemon::new().unwrap();

            for (group, devices) in targets.iter() {
                device.discover(group, devices, mdns.clone());
            }

            device.respond(ip, port, group.as_str(), mdns)
        })
    }
}
