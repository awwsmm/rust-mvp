use std::sync::Arc;
use std::time::Duration;

use uuid::Uuid;
use actuator_temperature::TemperatureActuator;

use controller::Controller;
use device::id::Id;
use device::name::Name;
use environment::Environment;
use sensor_temperature::TemperatureSensor;

fn main() {
    // the mDNS service daemon

    let mdns = Arc::new(mdns_sd::ServiceDaemon::new().unwrap());

    // in the local demo, all devices have the same ip (localhost)
    let ip = local_ip_address::local_ip().unwrap();

    // --------------------------------------------------------------------------------
    // spin up a sensor-actuator pair
    // --------------------------------------------------------------------------------

    // id has to be the same for the sensor and its corresponding actuator, name does not
    let id = Id::new(&Uuid::new_v4().to_string());
    let name = Name::new("My Thermo-5000");

    // here is the Sensor
    TemperatureSensor::start_new(ip, 8787, id.clone(), name.clone(), Arc::clone(&mdns));

    // here is the Actuator
    TemperatureActuator::start_new(ip, 9898, id.clone(), name.clone(), Arc::clone(&mdns));

    // --------------------------------------------------------------------------------
    // spin up the controller
    // --------------------------------------------------------------------------------

    let controller_port = 6565;
    Controller::start_default(ip, controller_port, Arc::clone(&mdns));

    // --------------------------------------------------------------------------------
    // spin up the controller
    // --------------------------------------------------------------------------------

    let environment_port = 5454;
    Environment::start_default(ip, environment_port, mdns);

    // demo should loop continually
    std::thread::sleep(Duration::MAX)
}
