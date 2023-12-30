use std::collections::HashMap;
use std::net::{IpAddr, TcpStream};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use mdns_sd::ServiceInfo;

use datum::Datum;
use device::id::Id;
use device::message::Message;
use device::model::Model;
use device::name::Name;
use device::{Device, Handler};

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
    address: String,
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

    fn get_address(&self) -> &String {
        &self.address
    }

    // TODO Controller should respond to HTTP requests from Sensors.
    fn get_handler(&self) -> Handler {
        let sender_name = self.get_name().to_string().clone();
        let sender_address = self.get_address().clone();

        let assessors = Arc::clone(&self.state.assessors);
        let assessors = assessors.lock();
        let assessors = assessors.unwrap().clone();

        let actuators = Arc::clone(&self.state.actuators);
        let actuators = actuators.lock();
        let actuators = actuators.unwrap().clone();

        Box::new(move |stream| {
            if let Ok(request) =
                Self::ack_and_parse_request(sender_name.clone(), sender_address.clone(), stream)
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
                        .or_else(|| DEFAULT_ASSESSOR.get(model.id().as_str()))
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
                                        let actuator = <Self as Device>::extract_address(actuator);
                                        println!("[Sensor] connecting to Actuator @ {}", actuator);

                                        let mut stream = TcpStream::connect(actuator).unwrap();

                                        let command = Message::ping_with_body(sender_name.clone(), sender_address.clone(), Some(command.to_string()));

                                        println!("[Controller] sending Command to Actuator\nvvvvvvvvvv\n{}\n^^^^^^^^^^", command);

                                        command.send(&mut stream);

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

    fn start(ip: IpAddr, port: u16, id: Id, name: Name) -> JoinHandle<()> {
        let host = ip.clone().to_string();
        let address = <Self as Device>::address(host, port.to_string());

        std::thread::spawn(move || {
            let device = Self::new(id, name, address);

            let mut targets = HashMap::new();
            targets.insert("_sensor".into(), &device.state.sensors);
            targets.insert("_actuator".into(), &device.state.actuators);

            Controller::poll(&device);

            device.run(ip, port, "_controller", targets);
        })
    }
}

impl Controller {
    pub fn new(id: Id, name: Name, address: String) -> Self {
        Self {
            name,
            id,
            state: State::new(),
            address,
        }
    }

    pub fn start_default(ip: IpAddr, port: u16) -> JoinHandle<()> {
        Self::start(ip, port, Id::new("controller"), Name::new("Controller"))
    }

    fn is_supported(model: &Model) -> bool {
        DEFAULT_ASSESSOR.contains_key(model.id().as_str())
    }

    /// Pings the latest `Sensor` so that it can (asynchronously) send a response containing the latest `Datum`.
    pub fn ping_sensor(sender_name: String, sender_address: String, info: &ServiceInfo) {
        let address = <Self as Device>::extract_address(info);

        let mut tcp_stream = TcpStream::connect(address).unwrap();

        // send the minimum possible payload. We only want to ping the Sensor
        // see: https://stackoverflow.com/a/9734866
        let ping = Message::ping(sender_name, sender_address);
        ping.send(&mut tcp_stream);
    }

    pub fn poll(&self) -> JoinHandle<()> {
        let sensors_mutex = Arc::clone(&self.state.sensors);
        let sender_name = self.get_name().to_string().clone();
        let sender_address = self.get_address().clone();

        std::thread::spawn(move || {
            loop {
                // We put the locks in this inner scope so the lock is released at the end of the scope
                {
                    let sensors_lock = sensors_mutex.lock();
                    let sensors = sensors_lock.unwrap();

                    for (_id, service_info) in sensors.iter() {
                        if let Some(Ok(model)) = Self::extract_model(service_info) {
                            if Self::is_supported(&model) {
                                println!(
                                    "[Controller::poll] pinging sensor \"{}\"",
                                    service_info
                                        .get_property("name")
                                        .map(|p| p.val_str())
                                        .unwrap_or("<unknown>")
                                );
                                Self::ping_sensor(
                                    sender_name.clone(),
                                    sender_address.clone(),
                                    service_info,
                                );
                            } else {
                                println!("[poll] unsupported Model: {:?}", service_info)
                            }
                        } else {
                            println!("[poll] could not find property 'model' in ServiceInfo")
                        }
                    }
                }

                // When the lock_result is released, we pause for a second, so self.sensors isn't continually locked
                std::thread::sleep(Duration::from_secs(3))
            }
        })
    }
}
