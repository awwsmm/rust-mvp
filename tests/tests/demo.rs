use std::time::Duration;

use uuid::Uuid;

use actuator::Actuator;
use actuator_temperature::TemperatureActuator;
use controller::Controller;
use device::id::Id;
use device::name::Name;
use environment::Environment;
use sensor::Sensor;
use sensor_temperature::TemperatureSensor;

#[test]
// this basic integration test just checks that nothing panics when running the demo for 5 seconds
fn test_demo() {
    // in the local demo, all devices have the same ip (localhost)
    let ip = local_ip_address::local_ip().unwrap();

    // --------------------------------------------------------------------------------
    // spin up a sensor-actuator pair
    // --------------------------------------------------------------------------------

    // id has to be the same for the sensor and its corresponding actuator, name does not
    let id = Id::new(Uuid::new_v4());

    // here is the Sensor
    let sensor_port = 8787;
    TemperatureSensor::start(ip, sensor_port, id.clone(), Name::new("My Thermo-5000 Sensor"), "_sensor".into());

    // here is the Actuator
    TemperatureActuator::start(ip, 9898, id.clone(), Name::new("My Thermo-5000 Actuator"), "_actuator".into());

    // --------------------------------------------------------------------------------
    // spin up the controller and the environment
    // --------------------------------------------------------------------------------

    Controller::start(ip, 6565, Id::new("controller"), Name::new("Controller"), String::from("_controller"));

    let environment_port = 5454;
    Environment::start(
        ip,
        environment_port,
        Id::new("environment"),
        Name::new("Environment"),
        String::from("_environment"),
    );

    std::thread::sleep(Duration::from_secs(5))
}
