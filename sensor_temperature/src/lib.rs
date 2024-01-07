use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use mdns_sd::ServiceInfo;

use datum::kind::Kind;
use datum::unit::Unit;
use datum::Datum;
use device::id::Id;
use device::model::Model;
use device::name::Name;
use device::{Device, Handler};
use sensor::Sensor;

/// `TemperatureSensor` is an example implementation of `Sensor`.
pub struct TemperatureSensor {
    id: Id,
    name: Name,
    environment: Arc<Mutex<Option<ServiceInfo>>>,
    controller: Arc<Mutex<Option<ServiceInfo>>>,
    data: Arc<Mutex<VecDeque<Datum>>>,
}

impl Device for TemperatureSensor {
    fn get_name(&self) -> &Name {
        &self.name
    }

    fn get_id(&self) -> &Id {
        &self.id
    }

    fn get_model() -> Model {
        Model::Thermo5000
    }

    fn get_handler(&self) -> Handler {
        Sensor::get_handler(self)
    }
}

impl Sensor for TemperatureSensor {
    fn new(id: Id, name: Name) -> TemperatureSensor {
        TemperatureSensor {
            id,
            name,
            environment: Arc::new(Mutex::new(None)),
            controller: Arc::new(Mutex::new(None)),
            data: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    fn get_environment(&self) -> &Arc<Mutex<Option<ServiceInfo>>> {
        &self.environment
    }

    fn get_controller(&self) -> &Arc<Mutex<Option<ServiceInfo>>> {
        &self.controller
    }

    fn get_datum_value_type() -> Kind {
        Kind::Float
    }

    fn get_datum_unit() -> Unit {
        Unit::DegreesC
    }

    fn get_data(&self) -> &Arc<Mutex<VecDeque<Datum>>> {
        &self.data
    }
}

#[cfg(test)]
mod sensor_temperature_tests {
    use std::collections::HashMap;
    use std::net::IpAddr;

    use super::*;

    #[test]
    fn test_get_name() {
        let id = Id::new("myId");
        let name = Name::new("myName");
        let sensor = TemperatureSensor::new(id.clone(), name.clone());

        let actual = sensor.get_name();
        let expected = name;

        assert_eq!(actual, &expected)
    }

    #[test]
    fn test_get_id() {
        let id = Id::new("myId");
        let name = Name::new("myName");
        let sensor = TemperatureSensor::new(id.clone(), name.clone());

        let actual = sensor.get_id();
        let expected = id;

        assert_eq!(actual, &expected)
    }

    #[test]
    fn test_get_model() {
        let actual = TemperatureSensor::get_model();
        let expected = Model::Thermo5000;

        assert_eq!(actual, expected)
    }

    // ServiceInfo doesn't implement PartialEq, so we have to compare field-by-field...
    fn compare_service_info(actual: &ServiceInfo, expected: &ServiceInfo) {
        assert_eq!(actual.is_addr_auto(), expected.is_addr_auto());
        assert_eq!(actual.get_type(), expected.get_type());
        assert_eq!(actual.get_subtype(), expected.get_subtype());
        assert_eq!(actual.get_fullname(), expected.get_fullname());

        assert_eq!(actual.get_property("name"), expected.get_property("name"));
        assert_eq!(actual.get_property("id"), expected.get_property("id"));
        assert_eq!(actual.get_property("model"), expected.get_property("model"));

        assert_eq!(actual.get_hostname(), expected.get_hostname());
        assert_eq!(actual.get_port(), expected.get_port());
        assert_eq!(actual.get_addresses(), expected.get_addresses());
        assert_eq!(actual.get_addresses_v4(), expected.get_addresses_v4());
        assert_eq!(actual.get_host_ttl(), expected.get_host_ttl());
        assert_eq!(actual.get_other_ttl(), expected.get_other_ttl());
        assert_eq!(actual.get_priority(), expected.get_priority());
        assert_eq!(actual.get_weight(), expected.get_weight());
    }

    #[test]
    fn test_get_environment() {
        let sensor = TemperatureSensor::new(Id::new("myId"), Name::new("myName"));

        let expected = ServiceInfo::new("my_domain", "the_name", "a_host", IpAddr::from([1, 2, 3, 4]), 42, HashMap::new()).unwrap();

        {
            // write to `environment` directly
            let mut lock = sensor.environment.lock().unwrap();
            let _ = lock.insert(expected.clone());
        }

        // read from `environment` indirectly, via get_environment()
        let lock = sensor.get_environment().lock().unwrap();
        let actual = lock.as_ref().unwrap();

        compare_service_info(actual, &expected)
    }

    #[test]
    fn test_get_controller() {
        let sensor = TemperatureSensor::new(Id::new("myId"), Name::new("myName"));

        let expected = ServiceInfo::new("my_domain", "the_name", "a_host", IpAddr::from([1, 2, 3, 4]), 42, HashMap::new()).unwrap();

        {
            // write to `controller` directly
            let mut lock = sensor.controller.lock().unwrap();
            let _ = lock.insert(expected.clone());
        }

        // read from `controller` indirectly, via get_controller()
        let lock = sensor.get_controller().lock().unwrap();
        let actual = lock.as_ref().unwrap();

        compare_service_info(actual, &expected)
    }

    #[test]
    fn test_get_datum_value_type() {
        let actual = TemperatureSensor::get_datum_value_type();
        let expected = Kind::Float;
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_get_datum_unit() {
        let actual = TemperatureSensor::get_datum_unit();
        let expected = Unit::DegreesC;
        assert_eq!(actual, expected)
    }

    #[test]
    fn test_get_data() {
        let sensor = TemperatureSensor::new(Id::new("myId"), Name::new("myName"));

        let expected = Datum::new_now(42.0, Unit::DegreesC);

        {
            // write to `data` directly
            let mut lock = sensor.data.lock().unwrap();
            lock.push_front(expected.clone());
        }

        // read from `data` indirectly, via get_data()
        let lock = sensor.get_data().lock().unwrap();
        let actual = lock.iter().next().unwrap();

        assert_eq!(actual, &expected)
    }
}
