use std::collections::HashMap;
use std::fmt::Display;
use std::net::{IpAddr, TcpStream};
use std::thread::JoinHandle;
use std::time::Duration;

use mdns_sd::{ServiceDaemon, ServiceInfo};

use device::message::Message;
use device::{Device, Handler};
use device::address::Address;
use device::id::Id;
use device::name::Name;

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
                    let sender_address = self.get_address();

                    let handler: Handler = Box::new(move |stream| {
                        if let Ok(request) = Self::ack_and_parse_request(
                            sender_name.as_str(),
                            sender_address,
                            stream,
                        ) {
                            if request.headers.get("sender_name")
                                == Some(&String::from("Controller"))
                            {
                                println!("[Actuator] received request from Controller\nvvvvvvvvvv\n{}\n^^^^^^^^^^", request);

                                let env = <Self as Device>::extract_address(&env).to_string();
                                println!("[Actuator] connecting to Environment @ {}", env);
                                let mut stream = TcpStream::connect(env).unwrap();

                                let mut headers = HashMap::new();
                                headers.insert("id", self_id.as_str());
                                headers.insert("model", self_model.as_str());
                                headers.insert("mode", "command");

                                let request = Message::ping(sender_name.as_str(), sender_address)
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

    fn start(ip: IpAddr, port: u16, id: Id, name: Name, group: String) -> JoinHandle<()> {
        std::thread::spawn(move || {
            let device = Self::new(id, name, Address::new(ip, port));

            let targets = device.targets_by_group();

            let mdns = ServiceDaemon::new().unwrap();

            for (group, devices) in targets.iter() {
                device.discover(group, devices, mdns.clone());
            }

            device.respond(ip, port, group.as_str(), mdns)
        })
    }
}

pub trait Command: Display {}
