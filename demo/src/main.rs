use std::time::Duration;

use uuid::Uuid;

use actuator_temperature::TemperatureActuator;
use controller::Controller;
use device::id::Id;
use device::name::Name;
use device::Device;
use environment::Environment;
use sensor_temperature::TemperatureSensor;

fn main() {
    // in the local demo, all devices have the same ip (localhost)
    let ip = local_ip_address::local_ip().unwrap();

    // --------------------------------------------------------------------------------
    // spin up a sensor-actuator pair
    // --------------------------------------------------------------------------------

    // id has to be the same for the sensor and its corresponding actuator, name does not
    let id = Id::new(&Uuid::new_v4().to_string());
    let name = Name::new("My Thermo-5000");

    // here is the Sensor
    let sensor_port = 8787;
    TemperatureSensor::start(ip, sensor_port, id.clone(), name.clone());

    // here is the Actuator
    let actuator_port = 9898;
    TemperatureActuator::start(ip, actuator_port, id, name);

    // --------------------------------------------------------------------------------
    // spin up the controller
    // --------------------------------------------------------------------------------

    let controller_port = 6565;
    Controller::start_default(ip, controller_port);

    // --------------------------------------------------------------------------------
    // spin up the controller
    // --------------------------------------------------------------------------------

    let environment_port = 5454;
    Environment::start_default(ip, environment_port);

    // demo should loop continually
    std::thread::sleep(Duration::MAX)
}
