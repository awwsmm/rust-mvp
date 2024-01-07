use std::collections::{HashMap, VecDeque};
use std::io::Write;
use std::net::{IpAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use log::{debug, warn};
use mdns_sd::{ServiceDaemon, ServiceInfo};

use datum::kind::Kind;
use datum::unit::Unit;
use datum::Datum;
use device::id::Id;
use device::message::Message;
use device::name::Name;
use device::{Device, Handler};

/// A Sensor collects data from the Environment.
pub trait Sensor: Device {
    fn new(id: Id, name: Name) -> Self;

    fn get_environment(&self) -> &Arc<Mutex<Option<ServiceInfo>>>;

    fn get_controller(&self) -> &Arc<Mutex<Option<ServiceInfo>>>;

    fn get_datum_value_type() -> Kind;

    fn get_datum_unit() -> Unit;

    fn get_data(&self) -> &Arc<Mutex<VecDeque<Datum>>>;

    /// By default, a `Sensor` responds to any request with the latest `Datum`.
    fn get_handler(&self) -> Handler {
        let self_name = self.get_name().clone();

        // Anything which depends on self must be cloned outside of the |stream| lambda.
        // We cannot refer to `self` inside of this lambda.
        let self_data = Arc::clone(self.get_data());

        Box::new(move |stream| {
            if let Ok(message) = Message::read(stream) {
                if message.start_line == "GET /data HTTP/1.1" {
                    Self::handle_get_data(stream, &self_data)
                } else if message.start_line == "GET /datum HTTP/1.1" {
                    Self::handle_get_datum(stream, &self_data)
                } else {
                    let msg = format!("cannot parse request: {}", message.start_line);
                    Self::handler_failure(self_name.clone(), stream, msg.as_str())
                }
            } else {
                Self::handler_failure(self_name.clone(), stream, "unable to read Message from stream")
            }
        })
    }

    /// Describes how `GET /data` requests are handled by `Sensor`s.
    ///
    /// **Design Decision**: `tcp_stream` is of type `impl Write` rather than `TcpStream` because
    /// this is easier to test. We do not use any `TcpStream`-specific APIs in this method.
    fn handle_get_data(tcp_stream: &mut impl Write, data: &Arc<Mutex<VecDeque<Datum>>>) {
        // get all of the data in this Sensor's buffer
        //     ex: curl 10.12.50.26:5454/data

        let data = data.lock().unwrap();
        let data: Vec<String> = data.iter().map(|d| d.to_string()).collect();
        let data = data.join(",");
        let data = format!("[{}]", data);

        let response = Message::respond_ok().with_body(data);
        response.write(tcp_stream)
    }

    /// Describes how `GET /datum` requests are handled by `Sensor`s.
    ///
    /// **Design Decision**: `tcp_stream` is of type `impl Write` rather than `TcpStream` because
    /// this is easier to test. We do not use any `TcpStream`-specific APIs in this method.
    fn handle_get_datum(tcp_stream: &mut impl Write, data: &Arc<Mutex<VecDeque<Datum>>>) {
        // get the latest Datum from this Sensor's buffer
        //     ex: curl 10.12.50.26:5454/datum

        let data = data.lock().unwrap();
        let datum = data.iter().next().map(|d| d.to_string());
        let datum = format!("[{}]", datum.unwrap_or_default());

        let response = Message::respond_ok().with_body(datum);
        response.write(tcp_stream)
    }

    fn start(ip: IpAddr, port: u16, id: Id, name: Name, group: String) -> JoinHandle<()> {
        std::thread::spawn(move || {
            // --------------------------------------------------------------------------------
            // create Device and discover required Message targets
            // --------------------------------------------------------------------------------
            let device = Self::new(id, name);

            let mdns = ServiceDaemon::new().unwrap();

            device.discover_once("_controller", device.get_controller(), mdns.clone());
            device.discover_once("_environment", device.get_environment(), mdns.clone());

            // --------------------------------------------------------------------------------
            // ping the Environment at regular intervals to get latest data
            // --------------------------------------------------------------------------------

            let sleep_duration = Duration::from_millis(50);
            let buffer_size = 10;

            // Anything which depends on device must be cloned outside of the || lambda below.
            // We cannot refer to `device` inside of this lambda.
            let device_name = device.get_name().clone();
            let device_id = device.get_id().clone();
            let device_kind = Self::get_datum_value_type();
            let device_unit = Self::get_datum_unit();

            let data = Arc::clone(device.get_data());
            let environment = Arc::clone(device.get_environment());

            std::thread::spawn(move || {
                let url = format!("/datum/{}", device_id);

                let mut headers: HashMap<&str, String> = HashMap::new();
                headers.insert("kind", device_kind.to_string());
                headers.insert("unit", device_unit.to_string());

                let query = Message::request_get(url.as_str()).with_headers(headers);

                loop {
                    {
                        let environment = environment.lock().unwrap();

                        match environment.as_ref().map(Self::extract_address) {
                            None => {
                                warn!("[Sensor] {} could not find environment", device_name);
                            }
                            Some(address) => {
                                let mut stream = TcpStream::connect(address.to_string()).unwrap();

                                debug!("[Sensor] {} is querying environment for a Datum", device_name);
                                query.write(&mut stream);
                                let message = Message::read(&mut stream).unwrap();
                                let datum = Datum::parse(message.body.unwrap()).unwrap();

                                debug!("[Sensor] {} received a Datum from environment: {}", device_name, datum);

                                // enforce buffer length, then push, then process
                                // .lock() must go in an inner scope so it is _unlocked_ while are thread::sleep()-ing, below
                                let mut data = data.lock().unwrap();
                                if data.len() == buffer_size {
                                    data.pop_back();
                                }
                                data.push_front(datum.clone());
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
mod sensor_tests {
    use datum::unit::Unit;
    use device::model::Model;

    use super::*;

    struct TestSensor {
        id: Id,
        name: Name,
        environment: Arc<Mutex<Option<ServiceInfo>>>,
        controller: Arc<Mutex<Option<ServiceInfo>>>,
        data: Arc<Mutex<VecDeque<Datum>>>,
    }

    impl Sensor for TestSensor {
        fn new(id: Id, name: Name) -> Self {
            TestSensor {
                id,
                name,
                environment: Arc::new(Mutex::new(None)),
                controller: Arc::new(Mutex::new(None)),
                data: Arc::new(Mutex::new(VecDeque::new())),
            }
        }

        fn get_environment(&self) -> &Arc<Mutex<Option<ServiceInfo>>> {
            &self.environment
        }

        fn get_controller(&self) -> &Arc<Mutex<Option<ServiceInfo>>> {
            &self.controller
        }

        fn get_datum_value_type() -> Kind {
            Kind::Float
        }

        fn get_datum_unit() -> Unit {
            Unit::DegreesC
        }

        fn get_data(&self) -> &Arc<Mutex<VecDeque<Datum>>> {
            &self.data
        }
    }

    impl Device for TestSensor {
        fn get_name(&self) -> &Name {
            &self.name
        }

        fn get_id(&self) -> &Id {
            &self.id
        }

        fn get_model() -> Model {
            Model::Unsupported
        }

        fn get_handler(&self) -> Handler {
            Box::new(|_| ())
        }
    }

    #[test]
    fn test_handle_get_data() {
        let mut data = VecDeque::new();
        let datum1 = Datum::new_now(1.0, Unit::DegreesC);
        let datum2 = Datum::new_now(2.0, Unit::DegreesC);
        let datum3 = Datum::new_now(3.0, Unit::DegreesC);
        data.push_front(datum1.clone());
        data.push_front(datum2.clone());
        data.push_front(datum3.clone());

        let data = Arc::new(Mutex::new(data));

        let mut buffer = Vec::new();

        TestSensor::handle_get_data(&mut buffer, &data);

        let actual = String::from_utf8(buffer).unwrap();

        let json = [datum3, datum2, datum1].map(|e| e.to_string()).join(",");
        let json = format!("[{}]", json);

        let expected = [
            "HTTP/1.1 200 OK",
            "Content-Length: 229",
            "Content-Type: text/json; charset=utf-8",
            "",
            json.as_str(),
        ]
        .join("\r\n");

        assert_eq!(actual, format!("{}\r\n\r\n", expected))
    }

    #[test]
    fn test_handle_get_datum() {
        let mut data = VecDeque::new();
        let datum1 = Datum::new_now(1.0, Unit::DegreesC);
        let datum2 = Datum::new_now(2.0, Unit::DegreesC);
        let datum3 = Datum::new_now(3.0, Unit::DegreesC);
        data.push_front(datum1.clone());
        data.push_front(datum2.clone());
        data.push_front(datum3.clone());

        let data = Arc::new(Mutex::new(data));

        let mut buffer = Vec::new();

        TestSensor::handle_get_datum(&mut buffer, &data);

        let actual = String::from_utf8(buffer).unwrap();

        let json = datum3.to_string();
        let json = format!("[{}]", json);

        let expected = [
            "HTTP/1.1 200 OK",
            "Content-Length: 77",
            "Content-Type: text/json; charset=utf-8",
            "",
            json.as_str(),
        ]
        .join("\r\n");

        assert_eq!(actual, format!("{}\r\n\r\n", expected))
    }
}
