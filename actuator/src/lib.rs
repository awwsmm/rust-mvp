use std::collections::HashMap;
use std::fmt::Display;
use std::net::{IpAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use mdns_sd::{ServiceDaemon, ServiceInfo};

use device::address::Address;
use device::id::Id;
use device::message::Message;
use device::name::Name;
use device::{Device, Handler};

/// An Actuator mutates the Environment.
pub trait Actuator: Device {
    fn new(id: Id, name: Name, address: Address) -> Self;

    fn get_environment(&self) -> &Arc<Mutex<Option<ServiceInfo>>>;

    /// By default, an `Actuator` forwards all incoming requests to the `Environment`.
    fn default_handler(&self) -> Handler {
        // Anything which depends on self must be cloned outside of the |stream| lambda.
        // We cannot refer to `self` inside of this lambda.
        let self_name = self.get_name().clone();
        let self_id = self.get_id().clone();
        let self_model = Self::get_model();

        let environment = Arc::clone(self.get_environment());

        Box::new(move |stream| {
            if let Ok(message) = Message::read(stream) {
                if message.start_line == "POST /command HTTP/1.1" {
                    // send a Command to this Actuator (Command is in the body)
                    //     ex: curl 10.12.50.26:5454/command -d '{"name":"HeatTo","value":"25"}'

                    // match message.body.as_ref().map(Command::parse) {
                    //     Some(Ok(command)) => {
                    let environment = environment.lock().unwrap();

                    match environment.as_ref().map(Self::extract_address) {
                        Some(address) => {
                            println!("[Actuator] forwarding body {:?} as-is to environment @ {}", message.body, address);

                            let mut environment = TcpStream::connect(address.to_string()).unwrap();

                            let mut headers = HashMap::new();
                            headers.insert("id", self_id.to_string());
                            headers.insert("model", self_model.to_string());

                            // forward Command to Environment
                            let forwarded_command = message.with_headers(headers);
                            forwarded_command.write(&mut environment);

                            // ack request from Controller to close the socket
                            let ack = Message::ack();
                            ack.write(stream)
                        }
                        None => {
                            let msg = "could not find environment";
                            Self::handler_failure(self_name.clone(), stream, msg)
                        }
                    }
                    // }
                    //     _ => {
                    //         let msg = format!("cannot parse body as command: {:?}", message.body);
                    //         Self::handler_failure(self_name.clone(), stream, msg.as_str())
                    //     }
                    // }
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
            let device = Self::new(id, name, Address::new(ip, port));

            let mdns = ServiceDaemon::new().unwrap();

            device.discover_once("_environment", device.get_environment(), mdns.clone());

            device.respond(ip, port, group.as_str(), mdns)
        })
    }
}

pub trait Command: Display {}
