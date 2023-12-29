use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use mdns_sd::ServiceInfo;

use device::id::Id;

use crate::assessor::Assessor;

pub struct State {
    // histories: HashMap<Id, SensorHistory>,
    pub(crate) sensors: Arc<Mutex<HashMap<Id, ServiceInfo>>>,
    pub(crate) actuators: Arc<Mutex<HashMap<Id, ServiceInfo>>>,
    #[allow(dead_code)] // FIXME remove ASAP
    pub(crate) assessors: Arc<Mutex<HashMap<Id, Assessor>>>,
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

    // pub(crate) fn send_command(info: &ServiceInfo, message: &str) -> TcpStream {
    //     let address = format!(
    //         "{}:{}",
    //         info.get_hostname().trim_end_matches('.'),
    //         info.get_port()
    //     );
    //
    //     println!("[send_request] connecting to url {}", address);
    //
    //     let mut stream = TcpStream::connect(address).unwrap();
    //
    //     println!("[send_request] sending message: {}", message);
    //
    //     message.send(&stream);
    //
    //     stream
    // }

    // /// Attempts to get the latest `Datum` from the `Sensor` with the specified `Id`.
    // #[allow(dead_code)] // remove this ASAP
    // pub fn read_sensor(info: &ServiceInfo) -> Result<Datum, String> {
    //     // send the minimum possible payload. We basically just want to ping the Sensor
    //     // see: https://stackoverflow.com/a/9734866
    //     let message = "GET / HTTP/1.1\r\n\r\n";
    //
    //     let mut stream = State::send_command(info, message);
    //
    //     let mut response = String::new();
    //
    //     let response = match stream.read_to_string(&mut response) {
    //         Err(_) => Err(String::from("unable to read stream")),
    //         Ok(_) => Ok(response),
    //     };
    //
    //     println!(
    //         "[read_sensor] response from url {}:{}\nvvvvvvvvvv\n{}\n^^^^^^^^^^",
    //         info.get_hostname().trim_end_matches('.'),
    //         info.get_port(),
    //         response.clone().unwrap_or(String::from("<error>"))
    //     );
    //
    //     // parse the response and return it
    //     response.and_then(|r| match r.trim().lines().last() {
    //         None => Err(String::from("cannot read response body")),
    //         Some(line) => Datum::parse(line),
    //     })
    // }

    // #[allow(dead_code)] // remove this ASAP
    // pub fn command_actuator(info: &ServiceInfo, command_json: String) -> std::io::Result<()> {
    //     let content_type = "application/json";
    //     let content_length = command_json.len();
    //
    //     // Place the serialized command inside the POST payload
    //     let message = format!(
    //         "POST HTTP/1.1\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
    //         content_type, content_length, command_json
    //     );
    //
    //     let response = State::send_request(info, message.as_str());
    //
    //     println!(
    //         "[command_actuator] response from url {}:{}\nvvvvvvvvvv\n{}\n^^^^^^^^^^",
    //         info.get_hostname().trim_end_matches('.'),
    //         info.get_port(),
    //         response.unwrap_or(String::from("<error>"))
    //     );
    //
    //     Ok(())
    // }
}
