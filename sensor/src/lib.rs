use std::collections::HashMap;
use std::io::Write;

use datum::Datum;
use device::message::Message;
use device::{Device, Handler};

/// A Sensor collects data from the Environment.
pub trait Sensor: Device {
    /// To get data out of a sensor, we call `get_datum()`.
    ///
    /// In the "real world", this would poll some actual physical sensor for a data point.
    ///
    /// In our example MVP, this queries the `Environment` for data.
    fn get_datum() -> Datum;

    /// By default, a `Sensor` responds to any request with the latest `Datum`.
    fn get_handler(&self) -> Handler {
        let address = self.get_address().clone();

        Box::new(move |stream, _mdns| {
            if let Ok(request) = Self::parse_http_request(stream) {
                println!("[Sensor] received\n----------\n{}\n----------", request);
                match request.headers.get("sender") {
                    None => {
                        println!("[Sensor] unable to handle request, cannot resolve sender")
                    }
                    Some(controller) if *controller == "controller" => {
                        println!("[Sensor] handling request from Controller");
                        let contents = Self::get_datum().to_string();
                        let response = Message::respond_ok_with_body(
                            address.clone(),
                            HashMap::new(),
                            contents.as_str(),
                        );

                        println!("[Sensor] sending response to Controller {}", response);

                        stream.write_all(response.to_string().as_bytes()).unwrap();
                    }
                    Some(other) => {
                        println!("[Sensor] ignoring request from {}", other)
                    }
                }
            }
        })
    }

    fn get_group() -> String {
        String::from("_sensor")
    }
}
