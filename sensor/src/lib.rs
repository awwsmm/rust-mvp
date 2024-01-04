use std::collections::{HashMap, VecDeque};
use std::net::{IpAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;

use mdns_sd::{ServiceDaemon, ServiceInfo};

use datum::kind::Kind;
use datum::unit::Unit;
use datum::Datum;
use device::address::Address;
use device::id::Id;
use device::message::Message;
use device::name::Name;
use device::{Device, Handler};

/// A Sensor collects data from the Environment.
pub trait Sensor: Device {
    fn new(id: Id, name: Name, address: Address) -> Self;

    fn get_environment(&self) -> &Arc<Mutex<Option<ServiceInfo>>>;

    fn get_controller(&self) -> &Arc<Mutex<Option<ServiceInfo>>>;

    fn get_datum_value_type(&self) -> Kind;

    fn get_datum_unit(&self) -> Unit;

    fn get_data(&self) -> &Arc<Mutex<VecDeque<Datum>>>;

    /// By default, a `Sensor` responds to any request with the latest `Datum`.
    fn default_handler(&self) -> Handler {
        let self_name = self.get_name().clone();

        // Anything which depends on self must be cloned outside of the |stream| lambda.
        // We cannot refer to `self` inside of this lambda.
        let self_data = Arc::clone(self.get_data());

        Box::new(move |stream| {
            if let Ok(message) = Message::read(stream) {
                if message.start_line == "GET /data HTTP/1.1" {
                    // get all of the data in this Sensor's buffer
                    //     ex: curl 10.12.50.26:5454/data

                    let data = self_data.lock().unwrap();
                    let data: Vec<String> = data.iter().map(|d| d.to_string()).collect();
                    let data = data.join("\r\n");

                    let response = Message::respond_ok().with_body(data);
                    response.write(stream)
                } else if message.start_line == "GET /datum HTTP/1.1" {
                    // get the latest Datum from this Sensor's buffer
                    //     ex: curl 10.12.50.26:5454/datum

                    let data = self_data.lock().unwrap();
                    let datum = data.iter().next().map(|d| d.to_string());

                    let response = match datum {
                        None => Message::respond_not_found().with_body("no data"),
                        Some(datum) => Message::respond_ok().with_body(datum),
                    };

                    response.write(stream)
                } else {
                    let msg = format!("cannot parse request: {}", message.start_line);
                    Self::handler_failure(self_name.clone(), stream, msg.as_str())
                }
            } else {
                Self::handler_failure(self_name.clone(), stream, "unable to read Message from stream")
            }
        })
    }

    fn start(ip: IpAddr, port: u16, id: Id, name: Name, group: String) -> JoinHandle<()> {
        std::thread::spawn(move || {
            // --------------------------------------------------------------------------------
            // create Device and discover required Message targets
            // --------------------------------------------------------------------------------
            let device = Self::new(id, name, Address::new(ip, port));

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
            let device_kind = device.get_datum_value_type();
            let device_unit = device.get_datum_unit();

            let data = Arc::clone(device.get_data());
            let environment = Arc::clone(device.get_environment());

            std::thread::spawn(move || {
                let url = format!("/datum/{}", device_id);

                let mut headers: HashMap<&str, String> = HashMap::new();
                headers.insert("kind", device_kind.to_string());
                headers.insert("unit", device_unit.to_string());

                let query = Message::request("GET", url.as_str()).with_headers(headers);

                loop {
                    {
                        let environment = environment.lock().unwrap();

                        match environment.as_ref().map(Self::extract_address) {
                            None => {
                                println!("[Sensor] {} could not find environment", device_name);
                            }
                            Some(address) => {
                                let mut stream = TcpStream::connect(address.to_string()).unwrap();

                                println!("[Sensor] {} is querying environment for a Datum", device_name);
                                query.write(&mut stream);
                                let message = Message::read(&mut stream).unwrap();
                                let datum = Datum::parse(message.body.unwrap()).unwrap();

                                println!("[Sensor] {} received a Datum from environment: {}", device_name, datum);

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
