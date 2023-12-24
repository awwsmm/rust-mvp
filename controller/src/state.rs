use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use mdns_sd::ServiceInfo;

use datum::Datum;
use device::id::Id;
use device::model::Model;

use crate::assessor::{Assessor, DEFAULT_ASSESSOR};

pub struct State {
    // histories: HashMap<Id, SensorHistory>,
    sensors: Arc<Mutex<HashMap<Id, ServiceInfo>>>,
    actuators: Arc<Mutex<HashMap<Id, ServiceInfo>>>,
    assessors: Arc<Mutex<HashMap<Id, Assessor>>>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            // histories: HashMap::new(),
            sensors: Arc::new(Mutex::new(HashMap::new())),
            actuators: Arc::new(Mutex::new(HashMap::new())),
            assessors: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl State {
    pub fn new() -> Self {
        Self::default()
    }

    fn is_supported(model: &Model) -> bool {
        DEFAULT_ASSESSOR.contains_key(model.to_string().as_str())
    }

    fn extract_id(info: &ServiceInfo) -> Id {
        let id = info.get_property("id").unwrap().to_string();
        let id = id.trim_start_matches("id=");
        Id::new(id)
    }

    fn extract_model(info: &ServiceInfo) -> Model {
        let model = info.get_property("model").unwrap().to_string();
        let model = model.trim_start_matches("model=");
        Model::parse(model).unwrap()
    }

    pub fn discover_sensors(&self) -> JoinHandle<()> {
        self.discover("_sensor")
    }

    pub fn discover_actuators(&self) -> JoinHandle<()> {
        self.discover("_actuator")
    }

    /// Creates a new thread to continually discover devices on the network in the specified group.
    fn discover(&self, group: &str) -> JoinHandle<()> {
        let devices = match group {
            "_sensor" => &self.sensors,
            "_actuator" => &self.actuators,
            _ => panic!("can only discover _sensor or _actuator, not {}", group),
        };

        let group = String::from(group);

        // clone the Arc<Mutex<>> around the devices so we can update them in multiple threads
        let devices_mutex = Arc::clone(devices);

        std::thread::spawn(move || {
            let mdns = mdns_sd::ServiceDaemon::new().unwrap();
            let service_type = format!("{}._tcp.local.", group);
            let receiver = mdns.browse(service_type.as_str()).unwrap();

            while let Ok(event) = receiver.recv() {
                if let mdns_sd::ServiceEvent::ServiceResolved(info) = event {
                    let id = State::extract_id(&info);
                    let model = State::extract_model(&info);

                    if Self::is_supported(&model) {
                        let devices_lock = devices_mutex.lock();
                        let mut devices_guard = devices_lock.unwrap();
                        devices_guard.insert(id, info);
                    } else {
                        println!("Found unsupported model {}", model)
                    }
                }
            }
        })
    }

    /// Connects to an address, sends the specified request, and returns the response, or an error
    #[allow(dead_code)] // remove this ASAP
    fn send_request(info: &ServiceInfo, request: &str) -> Result<String, String> {
        let address = format!(
            "{}:{}",
            info.get_hostname().trim_end_matches('.'),
            info.get_port()
        );

        println!("[send_request] connecting to url {}", address);

        let mut stream = TcpStream::connect(address).unwrap();

        println!("[send_request] sending request: {}", request);

        stream.write_all(request.as_bytes()).unwrap();

        let mut response = String::new();

        match stream.read_to_string(&mut response) {
            Err(_) => Err(String::from("unable to read stream")),
            Ok(_) => Ok(response),
        }
    }

    fn send_command(info: &ServiceInfo, request: &str) {
        let address = format!(
            "{}:{}",
            info.get_hostname().trim_end_matches('.'),
            info.get_port()
        );

        println!("[send_request] connecting to url {}", address);

        let mut stream = TcpStream::connect(address).unwrap();

        println!("[send_request] sending request: {}", request);

        stream.write_all(request.as_bytes()).unwrap()
    }

    /// Attempts to get the latest `Datum` from the `Sensor` with the specified `Id`.
    #[allow(dead_code)] // remove this ASAP
    pub fn read_sensor(info: &ServiceInfo) -> Result<Datum, String> {
        // send the minimum possible payload. We basically just want to ping the Sensor
        // see: https://stackoverflow.com/a/9734866
        let request = "GET / HTTP/1.1\r\n\r\n";

        let response = State::send_request(info, request);

        println!(
            "[read_sensor] response from url {}:{}\n----------\n{}\n----------",
            info.get_hostname().trim_end_matches('.'),
            info.get_port(),
            response.clone().unwrap_or(String::from("<error>"))
        );

        // parse the response and return it
        response.and_then(|r| match r.trim().lines().last() {
            None => Err(String::from("cannot read response body")),
            Some(line) => Datum::parse(line),
        })
    }

    #[allow(dead_code)] // remove this ASAP
    pub fn command_actuator(info: &ServiceInfo, command_json: String) -> std::io::Result<()> {
        let content_type = "application/json";
        let content_length = command_json.len();

        // Place the serialized command inside the POST payload
        let request = format!(
            "POST HTTP/1.1\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
            content_type, content_length, command_json
        );

        let response = State::send_request(info, request.as_str());

        println!(
            "[command_actuator] response from url {}:{}\n----------\n{}\n----------",
            info.get_hostname().trim_end_matches('.'),
            info.get_port(),
            response.unwrap_or(String::from("<error>"))
        );

        Ok(())
    }

    pub fn poll(&self) -> JoinHandle<()> {
        let sensors_mutex = Arc::clone(&self.sensors);
        let assessors = Arc::clone(&self.assessors);
        let actuators_mutex = Arc::clone(&self.actuators);

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
                        let datum = Self::read_sensor(service_info);
                        let model = Self::extract_model(service_info);

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
                                            Self::send_command(actuator, command_str.as_str());
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
