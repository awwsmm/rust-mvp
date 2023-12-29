use std::collections::HashMap;
use std::net::TcpStream;
use std::time::Duration;

use mdns_sd::ServiceInfo;

use datum::{Datum, DatumUnit, DatumValueType};
use device::message::Message;
use device::{Device, Handler};

/// A Sensor collects data from the Environment.
pub trait Sensor: Device {
    fn get_environment(&self) -> Option<ServiceInfo>;

    /// To get data out of a sensor, we call `get_datum()`.
    ///
    /// In the "real world", this would poll some actual physical sensor for a data point.
    ///
    /// In our example MVP, this queries the `Environment` for data.
    fn get_datum() -> Datum;

    /// By default, a `Sensor` responds to any request with the latest `Datum`.
    fn default_handler(&self) -> Handler {
        loop {
            // loop until there is an environment to forward requests to
            match self.get_environment() {
                None => {
                    println!(
                        "[Sensor] \"{}\" could not find environment",
                        self.get_name()
                    );
                    std::thread::sleep(Duration::from_secs(1));
                    continue;
                }
                Some(env) => {
                    println!(
                        "[Sensor] \"{}\" found \"{}\" at {}",
                        self.get_name(),
                        env.get_property("name")
                            .map(|p| p.val_str())
                            .unwrap_or("<unknown>"),
                        env.get_fullname()
                    );

                    let sender_name = self.get_name().to_string().clone();
                    let sender_address = self.get_address().clone();

                    let self_id = self.get_id().to_string();
                    let self_kind = self.get_datum_value_type().to_string();
                    let self_unit = self.get_datum_unit().to_string();

                    let handler: Handler = Box::new(move |stream| {
                        if let Ok(request) = Self::ack_and_parse_request(
                            sender_name.clone(),
                            sender_address.clone(),
                            stream,
                        ) {
                            if request.headers.get("sender_name")
                                == Some(&String::from("Controller"))
                            {
                                println!("[Sensor] received request from Controller\nvvvvvvvvvv\n{}\n^^^^^^^^^^", request);

                                let env = <Self as Device>::extract_address(&env);
                                println!("[Sensor] connecting to Environment @ {}", env);
                                let mut stream = TcpStream::connect(env).unwrap();

                                let mut headers = HashMap::new();
                                headers.insert("id".into(), self_id.clone());
                                headers.insert("kind".into(), self_kind.clone());
                                headers.insert("unit".into(), self_unit.clone());

                                let request = Message::ping_with_headers(
                                    sender_name.clone(),
                                    sender_address.clone(),
                                    headers,
                                );
                                println!("[Sensor] forwarding request to Environment\nvvvvvvvvvv\n{}\n^^^^^^^^^^", request);
                                request.send(&mut stream);
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

    fn get_group() -> String {
        String::from("_sensor")
    }

    fn get_datum_value_type(&self) -> DatumValueType;

    fn get_datum_unit(&self) -> DatumUnit;
}
