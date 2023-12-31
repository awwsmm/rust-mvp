use std::collections::HashMap;
use std::fmt::Display;
use std::net::TcpStream;
use std::time::Duration;

use mdns_sd::ServiceInfo;

use device::message::Message;
use device::{Device, Handler};

/// An Actuator mutates the Environment.
pub trait Actuator: Device {
    fn get_environment(&self) -> Option<ServiceInfo>;

    /// By default, an `Actuator` forwards all incoming requests to the `Environment`.
    fn default_handler(&self) -> Handler {
        loop {
            // loop until there is an environment to forward commands to
            match self.get_environment() {
                None => {
                    println!(
                        "[Actuator] \"{}\" could not find environment",
                        self.get_name()
                    );
                    std::thread::sleep(Duration::from_secs(1));
                    continue;
                }
                Some(env) => {
                    println!(
                        "[Actuator] \"{}\" found \"{}\" at {}",
                        self.get_name(),
                        env.get_property("name")
                            .map(|p| p.val_str())
                            .unwrap_or("<unknown>"),
                        env.get_fullname()
                    );

                    let self_id = self.get_id().to_string();
                    let self_model = Self::get_model().to_string();

                    let sender_name = self.get_name().to_string().clone();
                    let sender_address = self.get_address().clone();

                    let handler: Handler = Box::new(move |stream| {
                        if let Ok(request) = Self::ack_and_parse_request(
                            sender_name.as_str(),
                            sender_address.as_str(),
                            stream,
                        ) {
                            if request.headers.get("sender_name")
                                == Some(&String::from("Controller"))
                            {
                                println!("[Actuator] received request from Controller\nvvvvvvvvvv\n{}\n^^^^^^^^^^", request);

                                let env = <Self as Device>::extract_address(&env);
                                println!("[Actuator] connecting to Environment @ {}", env);
                                let mut stream = TcpStream::connect(env).unwrap();

                                let mut headers = HashMap::new();
                                headers.insert("id", self_id.as_str());
                                headers.insert("model", self_model.as_str());
                                headers.insert("mode", "command");

                                let request =
                                    Message::ping(sender_name.as_str(), sender_address.as_str())
                                        .with_headers(headers)
                                        .with_body(request.body.unwrap());

                                println!("[Actuator] forwarding request to Environment\nvvvvvvvvvv\n{}\n^^^^^^^^^^", request);

                                request.write(&mut stream);
                            } else {
                                println!("[Actuator] received request from unhandled sender '{:?}'. Ignoring.", request.headers.get("sender_name"));
                            }
                        }
                    });

                    break handler;
                }
            }
        }
    }

    fn get_handler(&self) -> Handler {
        self.default_handler()
    }
}

pub trait Command: Display {}
