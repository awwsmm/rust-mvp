use std::collections::HashMap;
use std::fmt::Display;
use std::io::Write;
use std::net::{IpAddr, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use mdns_sd::{ServiceDaemon, ServiceInfo};

use device::id::Id;
use device::message::Message;
use device::model::Model;
use device::name::Name;
use device::{Device, Handler};

/// An Actuator mutates the Environment.
pub trait Actuator: Device {
    fn new(id: Id, name: Name) -> Self;

    fn get_environment(&self) -> &Arc<Mutex<Option<ServiceInfo>>>;

    /// By default, an `Actuator` forwards all incoming requests to the `Environment`.
    // coverage: off
    // routing can be verified by inspection
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
                    Self::handle_post_command(stream, &environment, message, &self_id, self_model, &self_name)
                } else {
                    let msg = format!("cannot parse request: {}", message.start_line);
                    Self::handler_failure(self_name.clone(), stream, msg.as_str())
                }
            } else {
                Self::handler_failure(self_name.clone(), stream, "unable to read Message from stream")
            }
        })
    }
    // coverage: on

    /// Describes how `POST /command` requests are handled by `Actuator`s.
    ///
    /// **Design Decision**: `tcp_stream` is of type `impl Write` rather than `TcpStream` because
    /// this is easier to test. We do not use any `TcpStream`-specific APIs in this method.
    // coverage: off
    // cannot be tested in a unit test because of `TcpStream::connect`
    fn handle_post_command(
        stream: &mut impl Write,
        environment: &Arc<Mutex<Option<ServiceInfo>>>,
        message: Message,
        self_id: &Id,
        self_model: Model,
        self_name: &Name,
    ) {
        // send a Command to this Actuator (Command is in the body)
        //     ex: curl 10.12.50.26:5454/command -d '{"name":"HeatBy","value":"25"}'

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
                let ack = Message::respond_ok();
                ack.write(stream)
            }
            None => {
                let msg = "could not find environment";
                Self::handler_failure(self_name.clone(), stream, msg)
            }
        }
    }
    // coverage: on

    // coverage: off
    // this is very difficult to test outside of an integration test
    fn start(ip: IpAddr, port: u16, id: Id, name: Name, group: String) -> JoinHandle<()> {
        std::thread::spawn(move || {
            let device = Self::new(id, name);

            let mdns = ServiceDaemon::new().unwrap();

            device.discover_once("_environment", device.get_environment(), mdns.clone());

            device.respond(ip, port, group.as_str(), mdns)
        })
    }
    // coverage: on
}

pub trait Command: Display {}
