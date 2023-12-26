use std::fmt::Display;
use std::io::Write;

use device::handler::Handler;
use device::message::Message;
use device::Device;

/// An Actuator mutates the Environment.
pub trait Actuator: Device {
    /// By default, an `Actuator` forwards all incoming requests to the `Environment`.
    fn default_handler() -> Handler {
        Handler::new(|stream| {
            if let Ok(message) = Self::parse_http_request(stream) {
                println!("[Actuator] received\n----------\n{}\n----------", message);

                let message = Message::ack();

                stream.write_all(message.to_string().as_bytes()).unwrap();

                // TODO forward command to Environment
            }
        })
    }

    fn get_handler(&self) -> Handler {
        Self::default_handler()
    }

    fn get_group() -> String {
        String::from("_actuator")
    }
}

pub trait Command: Display {}
