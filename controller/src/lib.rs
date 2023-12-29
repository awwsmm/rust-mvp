use std::collections::HashMap;
use std::net::{IpAddr, TcpStream};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use mdns_sd::{ServiceDaemon, ServiceInfo};

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

    fn get_group() -> String {
        String::from("_controller")
    }

    fn get_address(&self) -> &String {
        &self.address
    }

    fn get_handler(&self) -> Handler {
        // Box::new(move |stream, mdns| {
        //     if let Ok(message) = Self::parse_http_request(stream) {
        //         println!("Controller received message: {}", message);
        //     }
        // })
        Box::new(|_, _| ())
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
            println!(">>> [controller start] SPAWNED A NEW THREAD");
            let device = Self::new(id, name, address);

            let mut targets = HashMap::new();
            targets.insert("_sensor".into(), &device.state.sensors);
            targets.insert("_actuator".into(), &device.state.actuators);

            Controller::poll(&device);

            device.run(ip, port, "_controller", targets, mdns);
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

    pub fn start_default(ip: IpAddr, port: u16, mdns: Arc<ServiceDaemon>) -> JoinHandle<()> {
        Self::start(
            ip,
            port,
            Id::new("controller"),
            Name::new("controller"),
            mdns,
        )
    }

    fn is_supported(model: &Model) -> bool {
        DEFAULT_ASSESSOR.contains_key(model.id().as_str())
    }

    /// Pings the latest `Sensor` so that it can (asynchronously) send a response containing the latest `Datum`.
    pub fn ping_sensor(sender: String, info: &ServiceInfo) {
        let address = <Self as Device>::extract_address(info);

        let mut tcp_stream = TcpStream::connect(address).unwrap();

        // send the minimum possible payload. We only want to ping the Sensor
        // see: https://stackoverflow.com/a/9734866
        let ping = Message::ping(sender);
        ping.send(&mut tcp_stream);
    }

    pub fn poll(&self) -> JoinHandle<()> {
        let sensors_mutex = Arc::clone(&self.state.sensors);
        // let assessors = Arc::clone(&self.state.assessors);
        // let actuators_mutex = Arc::clone(&self.state.actuators);

        let address = self.get_address().clone();

        std::thread::spawn(move || {
            println!(">>> [poll] SPAWNED A NEW THREAD");

            loop {
                // We put the locks in this inner scope so the lock is released at the end of the scope
                {
                    let sensors_lock = sensors_mutex.lock();
                    let sensors = sensors_lock.unwrap();

                    println!("known sensors: {}", sensors.len());

                    // let actuators_lock = actuators_mutex.lock();
                    // let actuators = actuators_lock.unwrap();

                    // println!("known actuators: {}", actuators.len());

                    // let assessors = assessors.lock();
                    // let assessors = assessors.unwrap();

                    for (id, service_info) in sensors.iter() {
                        if let Some(Ok(model)) = Self::extract_model(service_info) {
                            if Self::is_supported(&model) {
                                println!("[poll] pinging sensor with id {}", id);
                                Self::ping_sensor(address.clone(), service_info);

                                // println!(
                                //     "[poll] assessing datum received from sensor (model={})",
                                //     model.name()
                                // );
                                //
                                // println!(
                                //     "available assessors: {:?}",
                                //     assessors.keys().map(|each| each.to_string())
                                // );
                                //
                                // if let Some(assessor) = assessors
                                //     .get(id)
                                //     .or_else(|| DEFAULT_ASSESSOR.get(model.id().as_str()))
                                // {
                                //     match datum {
                                //         Err(msg) => {
                                //             println!("unable to read sensor due to: {}", msg)
                                //         }
                                //         Ok(datum) => {
                                //             println!(
                                //                 "[poll] successfully received datum from sensor: {}",
                                //                 datum
                                //             );
                                //
                                //             let command = (assessor.assess)(&datum);
                                //
                                //             if let Some(command) = command {
                                //                 let command_str = command.to_string();
                                //
                                //                 println!(
                                //                     "[poll] sending command [{}] to actuator",
                                //                     command_str
                                //                 );
                                //
                                //                 if let Some(actuator) = actuators.get(id) {
                                //                     State::send_command(
                                //                         actuator,
                                //                         command_str.as_str(),
                                //                     );
                                //                 }
                                //             }
                                //         }
                                //     }
                                // } else {
                                //     println!(
                                //         "[poll] assessor does not contain id: {}\nknown ids: {:?}",
                                //         id,
                                //         assessors.keys()
                                //     )
                                // }
                            } else {
                                println!("[poll] unsupported Model: {}", model.name())
                            }
                        } else {
                            println!("[poll] could not find property 'model' in ServiceInfo")
                        }
                    }
                }

                // When the lock_result is released, we pause for a second, so self.sensors isn't continually locked
                std::thread::sleep(Duration::from_secs(1))
            }
        })
    }
}
