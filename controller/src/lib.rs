use std::collections::{HashMap, VecDeque};
use std::net::{IpAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use mdns_sd::ServiceDaemon;

use datum::Datum;
use device::address::Address;
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
    address: Address,
    data: Arc<Mutex<HashMap<Id, VecDeque<Datum>>>>,
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
        // Anything which depends on self must be cloned outside of the |stream| lambda.
        // We cannot refer to `self` inside of this lambda.
        let self_name = self.get_name().clone();
        let self_data = Arc::clone(&self.data);

        Box::new(move |stream| {
            if let Ok(message) = Message::read(stream) {
                if message.start_line == "GET /data HTTP/1.1" {
                    // get all of the data in this Controller's buffer, grouped by Sensor
                    //     ex: curl 10.12.50.26:5454/data

                    let data = self_data.lock().unwrap();
                    let sensors: Vec<String> = data
                        .iter()
                        .map(|(id, buffer)| {
                            let data: Vec<String> = buffer.iter().map(|d| d.to_string()).collect();
                            let data = data.join(",");
                            format!(r#"{{"id":"{}","data":[{}]}}"#, id, data)
                        })
                        .collect();
                    let body = format!("[{}]", sensors.join(","));

                    let response = Message::respond_ok().with_body(body);
                    response.write(stream)
                } else {
                    // TODO implement other endpoints
                    let msg = format!("cannot parse request: {}", message.start_line);
                    Self::handler_failure(self_name.clone(), stream, msg.as_str())
                }
            } else {
                Self::handler_failure(self_name.clone(), stream, "unable to read Message from stream")
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
            data: Arc::new(Mutex::new(HashMap::new())),
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

    pub fn start(ip: IpAddr, port: u16, id: Id, name: Name, group: String) -> JoinHandle<()> {
        std::thread::spawn(move || {
            // --------------------------------------------------------------------------------
            // create Device and discover required Message targets
            // --------------------------------------------------------------------------------

            let device = Self::new(id, name, Address::new(ip, port));

            let mut targets = HashMap::new();
            targets.insert("_sensor", Arc::clone(&device.state.sensors));
            targets.insert("_actuator", Arc::clone(&device.state.actuators));

            let mdns = ServiceDaemon::new().unwrap();

            for (group, devices) in targets.iter() {
                device.discover_continually(group, devices, mdns.clone());
            }
            // --------------------------------------------------------------------------------
            // ping the Sensors at regular intervals to get latest data
            // --------------------------------------------------------------------------------

            let sleep_duration = Duration::from_secs(1);
            let buffer_size = 10;

            let sensors = Arc::clone(&device.state.sensors);
            let data = Arc::clone(&device.data);
            let assessors = Arc::clone(&device.state.assessors);
            let actuators = Arc::clone(&device.state.actuators);

            std::thread::spawn(move || {
                let query = Message::request("GET", "/datum");

                loop {
                    {
                        let sensors = sensors.lock().unwrap();
                        let mut data = data.lock().unwrap();
                        let assessors = assessors.lock().unwrap();
                        let actuators = actuators.lock().unwrap();

                        for (id, info) in sensors.iter() {
                            let address = Self::extract_address(info);
                            let mut stream = TcpStream::connect(address.to_string()).unwrap();
                            let sensor_name = Self::extract_name(info).unwrap();
                            let sensor_model = Self::extract_model(info).unwrap().unwrap();

                            println!("[Controller] querying {} for a Datum", sensor_name);
                            query.write(&mut stream);
                            let message = Message::read(&mut stream).unwrap();

                            match Datum::parse(message.body.unwrap()) {
                                Ok(datum) => {
                                    println!("[Controller] received a Datum from {}: {}", sensor_name, datum);

                                    if !data.contains_key(id) {
                                        data.insert(id.clone(), VecDeque::new());
                                    }
                                    let buffer: &mut VecDeque<Datum> = data.get_mut(id).unwrap();

                                    // enforce buffer length, then save to buffer
                                    if buffer.len() == buffer_size {
                                        buffer.pop_back();
                                    }
                                    buffer.push_front(datum.clone());

                                    // assess new data point and (maybe) send Command to Actuator
                                    if let Some(assessor) = assessors.get(id).or_else(|| DEFAULT_ASSESSOR.get(sensor_model.to_string().as_str())) {
                                        match (assessor.assess)(&datum) {
                                            None => println!("[Controller] assessed Datum, but will not produce Command for Actuator"),
                                            Some(command) => {
                                                println!("[Controller] attempting to send Command to Actuator: {}", command);

                                                match actuators.get(id) {
                                                    None => println!("[Controller] cannot find Actuator with id: {}", id),
                                                    Some(actuator) => {
                                                        let actuator = <Self as Device>::extract_address(actuator).to_string();
                                                        println!("[Sensor] connecting to Actuator @ {}", actuator);

                                                        // TODO actually send command to Actuator

                                                        // let mut stream = TcpStream::connect(actuator).unwrap();
                                                        //
                                                        // let command = Message::ping(
                                                        //     sender_name.as_str(),
                                                        //     sender_address
                                                        // ).with_body(
                                                        //     command.to_string()
                                                        // );
                                                        //
                                                        // println!("[Controller] sending Command to Actuator\nvvvvvvvvvv\n{}\n^^^^^^^^^^", command);
                                                        //
                                                        // command.write(&mut stream);
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        println!("[Controller] assessor does not contain id: {}\nknown ids: {:?}", id, assessors.keys())
                                    }
                                }
                                Err(msg) => {
                                    println!("[Controller] received error: {}", msg)
                                }
                            }
                        }
                    }

                    std::thread::sleep(sleep_duration);
                }
            });

            // --------------------------------------------------------------------------------
            // respond to incoming requests
            // --------------------------------------------------------------------------------

            device.respond(ip, port, group.as_str(), mdns)
        })
    }
}
