use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use datum::Datum;
use device::Device;

/// A Sensor collects data from the Environment.
pub trait Sensor: Device {
    /// To get data out of a sensor, we call `sensor.get_datum()`.
    ///
    /// In the "real world", this would poll some actual physical sensor for a data point.
    ///
    /// In our example MVP, this queries the `Environment` for data.
    fn get_datum(&self) -> Datum;

    /// Responds to all incoming requests with the latest `Datum`.
    fn respond(&self, listener: TcpListener) {
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            let mut request = String::new();

            BufReader::new(&mut stream).read_line(&mut request).unwrap();
            println!("{} received request: {}", self.get_name(), request.trim());

            let contents = self.get_datum().to_string();
            let ack = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}\r\n\r\n",
                contents.len(),
                contents
            );
            stream.write_all(ack.as_bytes()).unwrap();
        }
    }
}

#[cfg(test)]
mod sensor_tests {
    use datum::{DatumUnit, DatumValue};
    use device::id::Id;
    use device::model::Model;
    use device::name::Name;

    use super::*;

    struct Thermometer {
        id: Id,
        name: Name,
        model: Model,
    }

    impl Thermometer {
        fn new() -> Thermometer {
            Thermometer {
                id: Id::new("should be random"),
                name: Name::new("Thermometer"),
                model: Model::Thermo5000,
            }
        }
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
    }

    impl Sensor for Thermometer {
        fn get_datum(&self) -> Datum {
            // in our example, this should query the Environment
            // in this test, we just return a constant value
            Datum::new_now(DatumValue::Float(42.0), DatumUnit::DegreesC)
        }
    }

    #[test]
    fn test_get_datum() {
        let thermometer = Thermometer::new();
        let datum = thermometer.get_datum();

        assert_eq!(datum.value, DatumValue::Float(42.0));
        assert_eq!(datum.unit, DatumUnit::DegreesC)
    }
}
