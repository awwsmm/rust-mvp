use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use device::handler::Handler;
use device::id::Id;
use device::model::Model;
use device::name::Name;
use device::Device;

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
    model: Model,
    id: Id,
    state: State,
}

impl Device for Controller {
    fn get_name(&self) -> &Name {
        &self.name
    }

    fn get_model(&self) -> &Model {
        &self.model
    }

    fn get_id(&self) -> &Id {
        &self.id
    }

    fn get_handler() -> Handler {
        Handler::new(|_| ())
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            name: Name::new("controller"),
            model: Model::Controller,
            id: Id::new("controller"),
            state: State::new(),
        }
    }
}

impl Controller {
    pub fn new() -> Controller {
        Controller::default()
    }

    /// Starts the discovery process as well as polling sensors
    pub fn start(&mut self) {
        // spawn a thread to look for sensors on the network continually
        self.discover("_sensor");

        // spawn a thread to look for actuators on the network continually
        self.discover("_actuator");

        // poll sensors for data in perpetuity, waiting 1s in between polls
        self.poll();

        // TODO I think we need two more loops
        //      Loop 1 should be another state-internal loop, polling all known sensors for data and saving it in the histories
        //      Loop 2 should be in this scope right here, and it should be the "control loop".
        //
        //      The "control loop" should
        //        1. get the latest Datums for each sensor
        //        2. determine if each sensor is outside of some user-defined range
        //        3. if so, command the sensor's corresponding actuator to perform some command

        // // Cycle through and poll the Sensors, if the return Datum is outside a defined range
        // // send a command off to the Actuator
        // let self_api_clone = Arc::clone(&self.state);
        //
        // // FIXME I think this loop below needs to happen inside the State, like the discover() loop
        // std::thread::spawn(move || loop {
        //
        //     // acquire a mutex lock on the state
        //     let mut ctrl = self_api_clone.lock().unwrap();
        //
        //     // loop over all known sensors
        //     for (id, )
        //
        //
        //
        //     // Create a temp vec to hold the data history as there is a lock on the controller and
        //     // we can't populate the history until the lock is released.
        //     let mut data_history: Vec<(Id, SensorHistory)> = Vec::new();
        //     {
        //         let ctrl = self_api_clone.lock().unwrap();
        //
        //     }
        //
        //     // Once we have exited the scope where we acquired the data and send commands
        //     // its safe to acquire lock on ctrl again and update its data history
        //     let mut ctrl = self_api_clone.lock().unwrap();
        //     for (id, history) in data_history {
        //         ctrl.data.insert(id, history);
        //     }
        //     std::thread::sleep(Duration::from_secs(5));
        // });
        //
        // Ok(())

        // run() should loop continually
        std::thread::sleep(Duration::MAX)
    }

    fn is_supported(model: &Model) -> bool {
        DEFAULT_ASSESSOR.contains_key(model.to_string().as_str())
    }

    /// Creates a new thread to continually discover devices on the network in the specified group.
    fn discover(&self, group: &str) -> JoinHandle<()> {
        let devices = match group {
            "_sensor" => &self.state.sensors,
            "_actuator" => &self.state.actuators,
            _ => panic!("can only discover _sensor or _actuator, not {}", group),
        };

        <Self as Device>::discover(group, devices, Self::is_supported)

        // let group = String::from(group);
        //
        // // clone the Arc<Mutex<>> around the devices so we can update them in multiple threads
        // let devices_mutex = Arc::clone(devices);
        //
        // std::thread::spawn(move || {
        //     let mdns = mdns_sd::ServiceDaemon::new().unwrap();
        //     let service_type = format!("{}._tcp.local.", group);
        //     let receiver = mdns.browse(service_type.as_str()).unwrap();
        //
        //     while let Ok(event) = receiver.recv() {
        //         if let mdns_sd::ServiceEvent::ServiceResolved(info) = event {
        //             let id = State::extract_id(&info);
        //             let model = State::extract_model(&info);
        //
        //             if Self::is_supported(&model) {
        //                 let devices_lock = devices_mutex.lock();
        //                 let mut devices_guard = devices_lock.unwrap();
        //                 devices_guard.insert(id, info);
        //             } else {
        //                 println!("Found unsupported model {}", model)
        //             }
        //         }
        //     }
        // })
    }

    pub fn poll(&self) -> JoinHandle<()> {
        let sensors_mutex = Arc::clone(&self.state.sensors);
        let assessors = Arc::clone(&self.state.assessors);
        let actuators_mutex = Arc::clone(&self.state.actuators);

        std::thread::spawn(move || {
            loop {
                // We put the locks in this inner scope so the lock is released at the end of the scope
                {
                    let sensors_lock = sensors_mutex.lock();
                    let sensors = sensors_lock.unwrap();

                    println!("known sensors: {}", sensors.len());

                    let actuators_lock = actuators_mutex.lock();
                    let actuators = actuators_lock.unwrap();

                    println!("known actuators: {}", actuators.len());

                    let assessors = assessors.lock();
                    let assessors = assessors.unwrap();

                    for (id, service_info) in sensors.iter() {
                        println!("[poll] polling sensor with id {}", id);
                        let datum = State::read_sensor(service_info);
                        let model = <Self as Device>::extract_model(service_info);

                        println!(
                            "[poll] assessing datum received from sensor (model={})",
                            model
                        );

                        println!(
                            "available assessors: {:?}",
                            assessors.keys().map(|each| each.to_string())
                        );

                        if let Some(assessor) = assessors
                            .get(id)
                            .or_else(|| DEFAULT_ASSESSOR.get(model.to_string().as_str()))
                        {
                            match datum {
                                Err(msg) => println!("unable to read sensor due to: {}", msg),
                                Ok(datum) => {
                                    println!(
                                        "[poll] successfully received datum from sensor: {}",
                                        datum
                                    );

                                    let command = (assessor.assess)(&datum);

                                    if let Some(command) = command {
                                        let command_str = command.to_string();

                                        println!(
                                            "[poll] sending command [{}] to actuator",
                                            command_str
                                        );

                                        if let Some(actuator) = actuators.get(id) {
                                            State::send_command(actuator, command_str.as_str());
                                        }
                                    }
                                }
                            }
                        } else {
                            println!(
                                "[poll] assessor does not contain id: {}\nknown ids: {:?}",
                                id,
                                assessors.keys()
                            )
                        }
                    }
                }

                // When the lock_result is released, we pause for a second, so self.sensors isn't continually locked
                std::thread::sleep(Duration::from_secs(1))
            }
        })
    }
}
