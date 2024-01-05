use std::time::Duration;

use device::id::Id;
use device::name::Name;
use environment::Environment;

fn main() {
    // TODO these should be args
    let port = 5454;

    // these should not change
    let ip = local_ip_address::local_ip().unwrap();
    let id = Id::new("environment");
    let name = Name::new("Environment");
    let group = String::from("_environment");

    Environment::start(ip, port, id, name, group);
    println!("Environment is running...");
    std::thread::sleep(Duration::MAX)
}
