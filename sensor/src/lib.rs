use std::collections::HashMap;
use std::io::Write;

use datum::Datum;
use device::handler::Handler;
use device::message::Message;
use device::Device;

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
        Handler::new(|stream| {
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
}

#[cfg(test)]
mod sensor_tests {
    use datum::{DatumUnit, DatumValue};
    use device::handler::Handler;
    use device::id::Id;
    use device::model::Model;
    use device::name::Name;

    use super::*;

    struct Thermometer {
        id: Id,
        name: Name,
        model: Model,
    }

    impl Device for Thermometer {
        fn get_name(&self) -> &Name {
            &self.name
        }

        fn get_model(&self) -> &Model {
            &self.model
        }

        fn get_id(&self) -> &Id {
            &self.id
        }

        fn get_handler(&self) -> Handler {
            Handler::ignore()
        }
    }

    impl Sensor for Thermometer {
        fn get_datum() -> Datum {
            // in our example, this should query the Environment
            // in this test, we just return a constant value
            Datum::new_now(DatumValue::Float(42.0), DatumUnit::DegreesC)
        }
    }

    #[test]
    fn test_get_datum() {
        let datum = Thermometer::get_datum();

        assert_eq!(datum.value, DatumValue::Float(42.0));
        assert_eq!(datum.unit, DatumUnit::DegreesC)
    }
}
