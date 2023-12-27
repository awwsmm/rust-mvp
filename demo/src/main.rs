use std::collections::HashMap;
use std::sync::Arc;
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

    let id_clone = id.clone();
    let name_clone = name.clone();
    let clone = Arc::clone(&mdns);

    // here is the Sensor
    std::thread::spawn(move || {
        let mdns = clone;

        let sensor_port = 8787;

        let device = TemperatureSensor::new(id_clone.clone(), name_clone.clone());

        let mut targets = HashMap::new();
        targets.insert(String::from("_controller"), &device.env);

        for (group, devices) in targets.iter() {
            device.discover(group, devices, Arc::clone(&mdns));
        }

        device.register(ip, sensor_port, "_sensor", Arc::clone(&mdns));
        let listener = device.bind(ip, sensor_port);
        let handler = device.get_handler();

        let clone = Arc::clone(&mdns);

        let mdns = clone;
        for stream in listener.incoming() {
            let mut stream = stream.unwrap();
            (handler.handle)(&mut stream, Arc::clone(&mdns));
        }
    });

    // ----------

    // here is the Actuator
    let actuator_port = 9898;
    TemperatureActuator::start(ip, actuator_port, id, name, Arc::clone(&mdns));

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
