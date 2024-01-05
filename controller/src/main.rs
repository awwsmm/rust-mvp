use std::time::Duration;

use controller::Controller;
use device::id::Id;
use device::name::Name;

fn main() {
    // TODO these should be args
    let port = 6565;

    // these should not change
    let ip = local_ip_address::local_ip().unwrap();
    let id = Id::new("controller");
    let name = Name::new("Controller");
    let group = String::from("_controller");
    let container_mode = true;

    Controller::start(ip, port, id, name, group, container_mode);
    println!("Controller is running...");
    std::thread::sleep(Duration::MAX)
}
