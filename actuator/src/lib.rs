use std::fmt::Display;
use std::net::TcpStream;
use std::time::Duration;

use mdns_sd::ServiceInfo;

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
                    println!("[Actuator] could not find Environment");
                    std::thread::sleep(Duration::from_secs(1));
                    continue;
                }
                Some(env) => {
                    println!("[Actuator] found Environment at {}", env.get_fullname());

                    let sender_name = self.get_name().to_string().clone();
                    let sender_address = self.get_address().clone();

                    let handler: Handler = Box::new(move |stream| {
                        if let Ok(request) = Self::ack_and_parse_request(
                            sender_name.clone(),
                            sender_address.clone(),
                            stream,
                        ) {
                            if request.headers.get("sender_name")
                                == Some(&String::from("controller"))
                            {
                                println!("[Actuator] received request from Controller\n----------\n{}\n----------", request);

                                let env = <Self as Device>::extract_address(&env);
                                println!("[Actuator] connecting to Environment @ {}", env);
                                let mut stream = TcpStream::connect(env).unwrap();

                                println!(
                                    "[Actuator] forwarding message as-is to Environment: {}",
                                    request
                                );
                                request.send(&mut stream);
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

    fn get_group() -> String {
        String::from("_actuator")
    }
}

pub trait Command: Display {}
