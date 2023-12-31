use std::time::Duration;

use uuid::Uuid;

use actuator_temperature::TemperatureActuator;
use controller::Controller;
use device::address::Address;
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
    let id = Id::new(Uuid::new_v4());

    // here is the Sensor
    let sensor_port = 8787;
    <TemperatureSensor as Device>::start(
        ip,
        sensor_port,
        id.clone(),
        Name::new("My Thermo-5000 Sensor"),
        "_sensor".into(),
    );

    // here is the Actuator
    <TemperatureActuator as Device>::start(
        ip,
        9898,
        id.clone(),
        Name::new("My Thermo-5000 Actuator"),
        "_actuator".into(),
    );

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

    // we continually tell the Controller to poll the sensors
    loop {
        std::thread::sleep(Duration::from_secs(1));

        Controller::ping_sensor(
            "Controller", // FIXME this must be "Controller" or this does not work
            Address::new(ip, environment_port),
            Address::new(ip, sensor_port),
        );
    }
}
