use std::collections::HashMap;
use std::net::TcpStream;
use std::time::Duration;

use mdns_sd::ServiceInfo;

use datum::kind::Kind;
use datum::unit::Unit;
use device::message::Message;
use device::{Device, Handler};

/// A Sensor collects data from the Environment.
pub trait Sensor: Device {
    fn get_environment(&self) -> Option<ServiceInfo>;

    fn get_controller(&self) -> Option<ServiceInfo>;

    /// By default, a `Sensor` responds to any request with the latest `Datum`.
    fn default_handler(&self) -> Handler {
        loop {
            // loop until there is an environment to forward requests to
            match (self.get_environment(), self.get_controller()) {
                (None, _) => {
                    println!(
                        "[Sensor] \"{}\" could not find environment",
                        self.get_name()
                    );
                    std::thread::sleep(Duration::from_secs(1));
                    continue;
                }
                (_, None) => {
                    println!("[Sensor] \"{}\" could not find controller", self.get_name());
                    std::thread::sleep(Duration::from_secs(1));
                    continue;
                }
                (Some(env), Some(controller)) => {
                    println!(
                        "[Sensor] \"{}\" found \"{}\" at {}",
                        self.get_name(),
                        env.get_property("name")
                            .map(|p| p.val_str())
                            .unwrap_or("<unknown>"),
                        env.get_fullname()
                    );

                    let sender_name = self.get_name().to_string().clone();
                    let sender_address = self.get_address();

                    let self_id = self.get_id().to_string();
                    let self_model = Self::get_model().to_string();
                    let self_kind = self.get_datum_value_type().to_string();
                    let self_unit = self.get_datum_unit().to_string();

                    let handler: Handler = Box::new(move |stream| {
                        if let Ok(request) = Self::ack_and_parse_request(
                            sender_name.as_str(),
                            sender_address,
                            stream,
                        ) {
                            if request.headers.get("sender_name")
                                == Some(&String::from("Controller"))
                            {
                                println!("[Sensor] received request from Controller\nvvvvvvvvvv\n{}\n^^^^^^^^^^", request);

                                let env = <Self as Device>::extract_address(&env).to_string();
                                println!("[Sensor] connecting to Environment @ {}", env);
                                let mut stream = TcpStream::connect(env).unwrap();

                                let mut headers = HashMap::new();
                                headers.insert("id", self_id.as_str());
                                headers.insert("kind", self_kind.as_str());
                                headers.insert("unit", self_unit.as_str());
                                headers.insert("mode", "request");

                                let request = Message::ping(sender_name.as_str(), sender_address)
                                    .with_headers(headers);
                                println!("[Sensor] forwarding request to Environment\nvvvvvvvvvv\n{}\n^^^^^^^^^^", request);
                                request.write(&mut stream);
                            } else if request.headers.get("sender_name")
                                == Some(&String::from("Environment"))
                            {
                                println!("[Sensor] received request from Environment\nvvvvvvvvvv\n{}\n^^^^^^^^^^", request);

                                let controller =
                                    <Self as Device>::extract_address(&controller).to_string();
                                println!("[Sensor] connecting to Controller @ {}", controller);
                                let mut stream = TcpStream::connect(controller).unwrap();

                                let mut headers = HashMap::new();
                                headers.insert("id", self_id.as_str());
                                headers.insert("model", self_model.as_str());

                                let request = Message::ping(sender_name.as_str(), sender_address)
                                    .with_headers(headers)
                                    .with_body(request.body.unwrap());

                                println!("[Sensor] forwarding Datum to Controller\nvvvvvvvvvv\n{}\n^^^^^^^^^^", request);
                                request.write(&mut stream);
                            } else {
                                println!("[Sensor] received request from unhandled sender '{:?}'. Ignoring.", request.headers.get("sender_name"));
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

    fn get_datum_value_type(&self) -> Kind;

    fn get_datum_unit(&self) -> Unit;
}
