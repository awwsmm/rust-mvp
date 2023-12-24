use std::io::Write;
use std::net::TcpStream;

use datum::Datum;
use device::Device;

/// A Sensor collects data from the Environment.
pub trait Sensor: Device {
    /// To get data out of a sensor, we call `sensor.get_datum()`.
    ///
    /// In the "real world", this would poll some actual physical sensor for a data point.
    ///
    /// In our example MVP, this queries the `Environment` for data.
    fn get_datum() -> Datum;

    fn handle(stream: &mut TcpStream, get_datum: fn() -> Datum) {
        let request = Self::extract_request(stream);
        println!("received request: {}", request.trim());

        let contents = get_datum().to_string();
        let ack = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}\r\n\r\n",
            contents.len(),
            contents
        );

        stream.write_all(ack.as_bytes()).unwrap();
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

        fn get_handler() -> Handler {
            Handler::new(|_| ())
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
