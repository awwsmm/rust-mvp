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
    fn default_handler() -> Handler {
        Box::new(|stream, _mdns| {
            if let Ok(message) = Self::parse_http_request(stream) {
                println!("[Sensor] received\n----------\n{}\n----------", message);

                let contents = Self::get_datum().to_string();
                let message = Message::respond_ok_with_body(HashMap::new(), contents.as_str());

                stream.write_all(message.to_string().as_bytes()).unwrap();
            }
        })
    }

    fn get_handler(&self) -> Handler {
        Self::default_handler()
    }

    fn get_group() -> String {
        String::from("_sensor")
    }
}
