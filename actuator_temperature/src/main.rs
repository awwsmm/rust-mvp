use std::time::Duration;

use actuator::Actuator;
use actuator_temperature::TemperatureActuator;
use device::id::Id;
use device::name::Name;

fn main() {
    // TODO these should be args
    let port = 9898;
    let id = Id::new("thermo-5000");
    let name = Name::new("My Thermo-5000 Actuator");

    // these should not change
    let ip = local_ip_address::local_ip().unwrap();
    let group = String::from("_actuator");

    TemperatureActuator::start(ip, port, id, name, group);
    println!("TemperatureActuator is running...");
    std::thread::sleep(Duration::MAX)
}
