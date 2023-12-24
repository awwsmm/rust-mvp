use uuid::Uuid;

use actuator::Actuator;
use actuator_temperature::TemperatureActuator;
use controller::Controller;
use device::{Device, Id, Model, Name};
use sensor::Sensor;
use sensor_temperature::TemperatureSensor;

fn main() {
    // in the local demo, all devices have the same ip (localhost)
    let ip = local_ip_address::local_ip().unwrap();

    // --------------------------------------------------------------------------------
    // spin up a sensor-actuator pair
    // --------------------------------------------------------------------------------

    // id has to be the same for the sensor and its corresponding actuator
    let id = Id::new(&Uuid::new_v4().to_string());
    let name = Name::new("user-defined device name, like 'Kitchen Thermostat'");
    let model = Model::Thermo5000;

    // ---------- here is the sensor ----------

    let sensor_port = 8787;

    let sensor = TemperatureSensor::new(id.clone(), model, name.clone());
    let listener = sensor.bind(ip, sensor_port, "_sensor");

    std::thread::spawn(move || {
        sensor.respond(listener);
    });

    // ---------- here is the actuator ----------

    let actuator_port = 9898;

    let actuator = TemperatureActuator::new(id, model, name);
    let listener = actuator.bind(ip, actuator_port, "_actuator");

    std::thread::spawn(move || {
        actuator.respond(listener);
    });

    // --------------------------------------------------------------------------------
    // spin up the controller
    // --------------------------------------------------------------------------------

    Controller::new().run();
}
