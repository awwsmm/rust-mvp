use std::fmt::Display;
use std::io::Write;
use std::net::TcpStream;
use std::time::Duration;

use mdns_sd::{ServiceEvent, ServiceInfo};

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
                    println!("!!! could not find environment");
                    std::thread::sleep(Duration::from_secs(1));
                    continue;
                }
                Some(info) => {
                    println!("!!! found environment at {}", info.get_fullname());

                    let sender = self.get_address().clone();

                    let handler: Handler = Box::new(move |stream, mdns| {
                        if let Ok(message) = Self::parse_http_request(stream) {
                            // respond to Controller with OK
                            println!("[Actuator] received\n----------\n{}\n----------", message);
                            let ack = Message::ack(sender.clone());
                            stream.write_all(ack.to_string().as_bytes()).unwrap();

                            // and forward command as-is to Environment
                            let service_type = "_environment._tcp.local.";
                            let receiver = mdns.browse(service_type).unwrap();

                            println!("FINDME about to enter while loop");

                            while let Ok(event) = receiver.recv() {
                                match event {
                                    ServiceEvent::SearchStarted(_) => {
                                        println!("FINDME -- found SearchStarted event")
                                    }
                                    ServiceEvent::ServiceFound(_, _) => {
                                        println!("FINDME -- found ServiceFound event")
                                    }
                                    ServiceEvent::ServiceResolved(_) => {
                                        println!("FINDME -- found ServiceResolved event")
                                    }
                                    ServiceEvent::ServiceRemoved(_, _) => {
                                        println!("FINDME -- found ServiceRemoved event")
                                    }
                                    ServiceEvent::SearchStopped(_) => {
                                        println!("FINDME -- found SearchStopped event")
                                    }
                                }

                                // if let mdns_sd::ServiceEvent::ServiceResolved(info) = event {
                                println!("[Actuator] found Environment, forwarding message");

                                let address = format!(
                                    "{}:{}",
                                    info.get_hostname().trim_end_matches('.'),
                                    info.get_port()
                                );

                                println!("[Actuator] connecting to url {}", address);

                                let mut stream = TcpStream::connect(address).unwrap();

                                println!("[Actuator] sending message: {}", message);

                                stream.write_all(message.to_string().as_bytes()).unwrap();
                            }

                            println!("FINDME exited while loop");
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
