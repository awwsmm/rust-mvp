use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use mdns_sd::ServiceInfo;

use datum::kind::Kind;
use datum::unit::Unit;
use datum::Datum;
use device::id::Id;
use device::model::Model;
use device::name::Name;
use device::{Device, Handler};
use sensor::Sensor;

pub struct TemperatureSensor {
    id: Id,
    name: Name,
    environment: Arc<Mutex<HashMap<Id, ServiceInfo>>>,
    controller: Arc<Mutex<HashMap<Id, ServiceInfo>>>,
    address: String,
}

impl Device for TemperatureSensor {
    fn get_name(&self) -> &Name {
        &self.name
    }

    fn get_id(&self) -> &Id {
        &self.id
    }

    fn get_model() -> Model {
        Model::Thermo5000
    }

    fn get_address(&self) -> &String {
        &self.address
    }

    fn get_handler(&self) -> Handler {
        self.default_handler()
    }

    fn start(ip: IpAddr, port: u16, id: Id, name: Name) -> JoinHandle<()> {
        let host = ip.clone().to_string();
        let address = <Self as Device>::address(host, port.to_string());

        std::thread::spawn(move || {
            let device = Self::new(id, name, address);

            let mut targets = HashMap::new();
            targets.insert("_controller".into(), &device.controller);
            targets.insert("_environment".into(), &device.environment);

            device.run(ip, port, "_sensor", targets);
        })
    }
}

impl Sensor for TemperatureSensor {
    fn get_environment(&self) -> Option<ServiceInfo> {
        let lock = self.environment.lock();
        let guard = lock.unwrap();
        guard.get(&Id::new("environment")).cloned()
    }

    fn get_controller(&self) -> Option<ServiceInfo> {
        let lock = self.controller.lock();
        let guard = lock.unwrap();
        guard.get(&Id::new("controller")).cloned()
    }

    fn get_datum() -> Datum {
        // TODO should query Environment
        Datum::new_now(25.0, Unit::DegreesC)
    }

    fn get_datum_value_type(&self) -> Kind {
        Kind::Float
    }

    fn get_datum_unit(&self) -> Unit {
        Unit::DegreesC
    }
}

impl TemperatureSensor {
    pub fn new(id: Id, name: Name, address: String) -> TemperatureSensor {
        TemperatureSensor {
            id,
            name,
            environment: Arc::new(Mutex::new(HashMap::new())),
            controller: Arc::new(Mutex::new(HashMap::new())),
            address,
        }
    }
}
