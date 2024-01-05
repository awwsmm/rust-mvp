use std::time::Duration;

use device::id::Id;
use device::name::Name;
use sensor::Sensor;
use sensor_temperature::TemperatureSensor;

fn main() {
    // TODO these should be args
    let port = 8787;
    let id = Id::new("thermo-5000");
    let name = Name::new("My Thermo-5000 Sensor");

    // these should not change
    let ip = local_ip_address::local_ip().unwrap();
    let group = String::from("_sensor");

    TemperatureSensor::start(ip, port, id, name, group);
    println!("TemperatureSensor is running...");
    std::thread::sleep(Duration::MAX)
}
