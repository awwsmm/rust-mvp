use std::collections::{HashMap, VecDeque};
use std::io::Write;
use std::net::{IpAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use log::{debug, error};
use mdns_sd::{ServiceDaemon, ServiceInfo};

use datum::Datum;
use device::address::Address;
use device::id::Id;
use device::message::Message;
use device::model::Model;
use device::name::Name;
use device::{Device, Handler};

use crate::assessor::{Assessor, DEFAULT_ASSESSOR};

mod assessor;

/// The Controller queries the `Sensor`s for `Datum`s and sends `Command`s to the `Actuator`s.
///
/// The Controller logically ties a `Sensor` to its corresponding `Actuator`. It queries the
/// `Sensor` for its data, and makes a decision based on its state and the `Sensor` data, then
/// (optionally) constructs an appropriate command to send to that `Sensor`'s `Actuator`.
///
/// The `Controller`'s state can be queried by an HTML frontend, so some historic data is held
/// in memory.
pub struct Controller {
    name: Name,
    id: Id,
    address: Address,
    container_mode: bool,
    sensors: Arc<Mutex<HashMap<Id, ServiceInfo>>>,
    actuators: Arc<Mutex<HashMap<Id, ServiceInfo>>>,
    assessors: Arc<Mutex<HashMap<Id, Assessor>>>,
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

    fn get_handler(&self) -> Handler {
        // Anything which depends on self must be cloned outside of the |stream| lambda.
        // We cannot refer to `self` inside of this lambda.
        let self_name = self.get_name().clone();
        let self_data = Arc::clone(&self.data);
        let self_address = self.address.to_string();
        let local_mode = self.container_mode;

        Box::new(move |stream| {
            if let Ok(message) = Message::read(stream) {
                if message.start_line == "GET /data HTTP/1.1" {
                    Self::handle_get_data(stream, &self_data)
                } else if message.start_line == "GET /datum HTTP/1.1" {
                    Self::handle_get_datum(stream, &self_data)
                } else if message.start_line == "GET /ui HTTP/1.1" {
                    Self::handle_get_ui(stream, local_mode, self_address.clone())
                } else {
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
    fn new(id: Id, name: Name, address: Address, container_mode: bool) -> Self {
        Self {
            name,
            id,
            address,
            container_mode,
            sensors: Arc::new(Mutex::new(HashMap::new())),
            actuators: Arc::new(Mutex::new(HashMap::new())),
            assessors: Arc::new(Mutex::new(HashMap::new())),
            data: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Describes how `GET /data` requests are handled by the `Controller`.
    ///
    /// **Design Decision**: `tcp_stream` is of type `impl Write` rather than `TcpStream` because
    /// this is easier to test. We do not use any `TcpStream`-specific APIs in this method.
    fn handle_get_data(tcp_stream: &mut impl Write, data: &Arc<Mutex<HashMap<Id, VecDeque<Datum>>>>) {
        // get all of the data in this Controller's buffer, grouped by Sensor
        //     ex: curl 10.12.50.26:5454/data

        let data = data.lock().unwrap();
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
        response.write(tcp_stream)
    }

    /// Describes how `GET /datum` requests are handled by the `Controller`.
    ///
    /// **Design Decision**: `tcp_stream` is of type `impl Write` rather than `TcpStream` because
    /// this is easier to test. We do not use any `TcpStream`-specific APIs in this method.
    fn handle_get_datum(tcp_stream: &mut impl Write, data: &Arc<Mutex<HashMap<Id, VecDeque<Datum>>>>) {
        // get the latest Datum in this Controller's buffer, grouped by Sensor
        //     ex: curl 10.12.50.26:5454/datum

        let data = data.lock().unwrap();
        let sensors: Vec<String> = data
            .iter()
            .map(|(id, buffer)| {
                let data = buffer.iter().next().map(|d| d.to_string());
                format!(r#"{{"id":"{}","datum":[{}]}}"#, id, data.unwrap_or_default())
            })
            .collect();
        let body = format!("[{}]", sensors.join(","));

        let response = Message::respond_ok().with_body(body);
        response.write(tcp_stream)
    }

    /// Describes how `GET /datum` requests are handled by the `Controller`.
    ///
    /// **Design Decision**: `tcp_stream` is of type `impl Write` rather than `TcpStream` because
    /// this is easier to test. We do not use any `TcpStream`-specific APIs in this method.
    fn handle_get_ui(tcp_stream: &mut impl Write, container_mode: bool, self_address: String) {
        let html = include_str!("index.html");

        let html = if container_mode {
            html.replace("192.168.2.16:6565", "localhost:6565")
        } else {
            html.replace("192.168.2.16:6565", self_address.as_str())
        };

        let mut headers = HashMap::new();
        headers.insert("Content-Type", "text/html; charset=utf-8");

        let response = Message::respond_ok().with_body(html).with_headers(headers);

        response.write(tcp_stream)
    }

    pub fn start(ip: IpAddr, port: u16, id: Id, name: Name, group: String, container_mode: bool) -> JoinHandle<()> {
        std::thread::spawn(move || {
            // --------------------------------------------------------------------------------
            // create Device and discover required Message targets
            // --------------------------------------------------------------------------------

            let device = Self::new(id, name, Address::new(ip, port), container_mode);

            let mut targets = HashMap::new();
            targets.insert("_sensor", Arc::clone(&device.sensors));
            targets.insert("_actuator", Arc::clone(&device.actuators));

            let mdns = ServiceDaemon::new().unwrap();

            for (group, devices) in targets.iter() {
                device.discover_continually(group, devices, mdns.clone());
            }
            // --------------------------------------------------------------------------------
            // ping the Sensors at regular intervals to get latest data
            // --------------------------------------------------------------------------------

            let sleep_duration = Duration::from_millis(50);
            let buffer_size = 500;

            let sensors = Arc::clone(&device.sensors);
            let data = Arc::clone(&device.data);
            let assessors = Arc::clone(&device.assessors);
            let actuators = Arc::clone(&device.actuators);

            std::thread::spawn(move || {
                let query = Message::request_get("/datum");

                // sleep just for a moment so the Sensor has a chance to grab its first Datum from the Environment
                std::thread::sleep(Duration::from_millis(100));

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

                            debug!("[Controller] querying {} for a Datum", sensor_name);
                            query.write(&mut stream);
                            let message = Message::read(&mut stream).unwrap();

                            match Datum::parse(message.body.unwrap().trim_start_matches('[').trim_end_matches(']')) {
                                Ok(datum) => {
                                    debug!("[Controller] received a Datum from {}: {}", sensor_name, datum);

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
                                            None => debug!("[Controller] assessed Datum, but will not produce Command for Actuator"),
                                            Some(command) => {
                                                debug!("[Controller] attempting to send Command to Actuator: {}", command);

                                                match actuators.get(id) {
                                                    None => error!("[Controller] cannot find Actuator with id: {}", id),
                                                    Some(actuator) => {
                                                        let actuator = <Self as Device>::extract_address(actuator).to_string();
                                                        debug!("[Controller] connecting to Actuator @ {}", actuator);
                                                        let mut stream = TcpStream::connect(actuator).unwrap();
                                                        let command = Message::request_post("/command").with_body((*command).to_string());
                                                        command.write(&mut stream);
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        error!("[Controller] assessor does not contain id: {}\nknown ids: {:?}", id, assessors.keys())
                                    }
                                }
                                Err(msg) => {
                                    error!("[Controller] received error: {}", msg)
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

#[cfg(test)]
mod controller_tests {
    use datum::unit::Unit;

    use super::*;

    #[test]
    fn test_get_name() {
        let expected = Name::new("myName");
        let address = Address::new(IpAddr::from([0, 0, 0, 0]), 10101);
        let container_mode = false;
        let controller = Controller::new(Id::new("myId"), expected.clone(), address, container_mode);
        let actual = controller.get_name();
        let expected = &expected;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_get_id() {
        let expected = Id::new("myId");
        let address = Address::new(IpAddr::from([0, 0, 0, 0]), 10101);
        let container_mode = false;
        let controller = Controller::new(expected.clone(), Name::new("myName"), address, container_mode);
        let actual = controller.get_id();
        let expected = &expected;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_get_model() {
        let actual = Controller::get_model();
        let expected = Model::Controller;
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_handle_get_data() {
        let id = Id::new("my_sensor");

        let mut data = VecDeque::new();
        let datum1 = Datum::new_now(1.0, Unit::DegreesC);
        let datum2 = Datum::new_now(2.0, Unit::DegreesC);
        let datum3 = Datum::new_now(3.0, Unit::DegreesC);
        data.push_front(datum1.clone());
        data.push_front(datum2.clone());
        data.push_front(datum3.clone());

        let mut all_data = HashMap::new();
        all_data.insert(id.clone(), data);
        let all_data = Arc::new(Mutex::new(all_data));

        let mut buffer = Vec::new();

        Controller::handle_get_data(&mut buffer, &all_data);

        let actual = String::from_utf8(buffer).unwrap();

        let json = [datum3, datum2, datum1].map(|e| e.to_string()).join(",");
        let json = format!(r#"[{{"id":"{}","data":[{}]}}]"#, id, json);

        let expected = [
            "HTTP/1.1 200 OK",
            "Content-Length: 257",
            "Content-Type: text/json; charset=utf-8",
            "",
            json.as_str(),
        ]
        .join("\r\n");

        assert_eq!(actual, format!("{}\r\n\r\n", expected))
    }

    #[test]
    fn test_handle_get_datum() {
        let id = Id::new("my_sensor");

        let mut data = VecDeque::new();
        let datum1 = Datum::new_now(1.0, Unit::DegreesC);
        let datum2 = Datum::new_now(2.0, Unit::DegreesC);
        let datum3 = Datum::new_now(3.0, Unit::DegreesC);
        data.push_front(datum1.clone());
        data.push_front(datum2.clone());
        data.push_front(datum3.clone());

        let mut all_data = HashMap::new();
        all_data.insert(id.clone(), data);
        let all_data = Arc::new(Mutex::new(all_data));

        let mut buffer = Vec::new();

        Controller::handle_get_datum(&mut buffer, &all_data);

        let actual = String::from_utf8(buffer).unwrap();

        let json = datum3.to_string();
        let json = format!(r#"[{{"id":"{}","datum":[{}]}}]"#, id, json);

        let expected = [
            "HTTP/1.1 200 OK",
            "Content-Length: 106",
            "Content-Type: text/json; charset=utf-8",
            "",
            json.as_str(),
        ]
        .join("\r\n");

        assert_eq!(actual, format!("{}\r\n\r\n", expected))
    }

    #[test]
    fn test_handle_get_ui() {
        let mut buffer = Vec::new();

        let address = String::from("1.2.3.4:5678");
        let container_mode = false;

        Controller::handle_get_ui(&mut buffer, container_mode, address.clone());

        let actual = String::from_utf8(buffer).unwrap();

        let html = include_str!("index.html").replace("192.168.2.16:6565", address.as_str());

        let expected = [
            "HTTP/1.1 200 OK",
            "Content-Length: 1837",
            "Content-Type: text/html; charset=utf-8",
            "",
            html.as_str(),
        ]
        .join("\r\n");

        assert_eq!(actual, format!("{}\r\n\r\n", expected))
    }
}
