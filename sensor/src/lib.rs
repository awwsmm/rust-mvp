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
use device::Device;

/// A Sensor collects data from the Environment.
pub trait Sensor: Device {
    fn new(id: Id, name: Name, address: Address) -> Self;

    fn get_environment(&self) -> &Arc<Mutex<Option<ServiceInfo>>>;

    fn get_controller(&self) -> &Arc<Mutex<Option<ServiceInfo>>>;

    fn get_environment_info(&self) -> Option<ServiceInfo>;

    fn get_controller_info(&self) -> Option<ServiceInfo>;

    fn get_datum_value_type(&self) -> Kind;

    fn get_datum_unit(&self) -> Unit;

    fn get_data(&self) -> &Arc<Mutex<VecDeque<Datum>>>;

    fn start(ip: IpAddr, port: u16, id: Id, name: Name, group: String) -> JoinHandle<()> {
        std::thread::spawn(move || {
            let device = Self::new(id, name, Address::new(ip, port));

            // Anything which depends on device must be cloned outside of the || lambda below.
            // We cannot refer to `device` inside of this lambda.
            let device_name = device.get_name().clone();
            let device_id = device.get_id().clone();
            let device_kind = device.get_datum_value_type();
            let device_unit = device.get_datum_unit();

            let mdns = ServiceDaemon::new().unwrap();

            device.discover_once("_controller", device.get_controller(), mdns.clone());
            device.discover_once("_environment", device.get_environment(), mdns.clone());

            let data = Arc::clone(device.get_data());
            let environment = Arc::clone(device.get_environment());

            // ping Environment at regular intervals to get the latest Datum
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
                            None => {}
                            Some(address) => {
                                let mut stream = TcpStream::connect(address.to_string()).unwrap();

                                println!(
                                    "[Sensor] {} is querying environment for a Datum",
                                    device_name
                                );
                                query.write(&mut stream);
                                let message = Message::read(&mut stream).unwrap();
                                let datum = Datum::parse(message.body.unwrap()).unwrap();

                                println!(
                                    "[Sensor] {} received a Datum from environment: {}",
                                    device_name, datum
                                );

                                // enforce buffer length, then push, then process
                                // .lock() must go in an inner scope so it is _unlocked_ while are thread::sleep()-ing, below
                                let mut data = data.lock().unwrap();
                                if data.len() == 10 {
                                    data.pop_back();
                                }
                                data.push_front(datum.clone());
                            }
                        }
                    }

                    std::thread::sleep(Duration::from_secs(1));
                }
            });

            device.respond(ip, port, group.as_str(), mdns)
        })
    }
}
