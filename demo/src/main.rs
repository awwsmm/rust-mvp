use std::collections::HashMap;
use uuid::Uuid;

use actuator_temperature::TemperatureActuator;
use controller::Controller;
use device::id::Id;
use device::model::Model;
use device::name::Name;
use device::Device;
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

    sensor.respond(ip, sensor_port, "_sensor");

    // ---------- here is the actuator ----------

    let actuator_port = 9898;

    let actuator = TemperatureActuator::new(id, model, name);

    let mut targets = HashMap::new();

    targets.insert("_controller".into(), &actuator.env);

    actuator.run(ip, actuator_port, "_actuator", targets);

    // --------------------------------------------------------------------------------
    // spin up the controller
    // --------------------------------------------------------------------------------

    let controller_port = 6565;

    Controller::new().start(ip, controller_port);
}
