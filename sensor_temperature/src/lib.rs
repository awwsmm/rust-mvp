use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use mdns_sd::{ServiceDaemon, ServiceInfo};

use datum::{Datum, DatumUnit};
use device::id::Id;
use device::model::Model;
use device::name::Name;
use device::{Device, Handler};
use sensor::Sensor;

pub struct TemperatureSensor {
    id: Id,
    name: Name,
    pub env: Arc<Mutex<HashMap<Id, ServiceInfo>>>,
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

    fn get_group() -> String {
        <Self as Sensor>::get_group()
    }

    fn get_address(&self) -> &String {
        &self.address
    }

    fn get_handler(&self) -> Handler {
        Sensor::get_handler(self)
    }

    fn start(
        ip: IpAddr,
        port: u16,
        id: Id,
        name: Name,
        mdns: Arc<ServiceDaemon>,
    ) -> JoinHandle<()> {
        let host = ip.clone().to_string();
        let address = <Self as Device>::address(host, port.to_string());

        std::thread::spawn(move || {
            println!(">>> [sensor_temp start] SPAWNED A NEW THREAD");

            let device = Self::new(id, name, address);

            let mut targets = HashMap::new();
            targets.insert("_controller".into(), &device.env);

            device.run(ip, port, "_sensor", targets, mdns);
        })
    }
}

impl Sensor for TemperatureSensor {
    fn get_datum() -> Datum {
        // TODO should query Environment
        Datum::new_now(25.0, DatumUnit::DegreesC)
    }
}

impl TemperatureSensor {
    pub fn new(id: Id, name: Name, address: String) -> TemperatureSensor {
        TemperatureSensor {
            id,
            name,
            env: Arc::new(Mutex::new(HashMap::new())),
            address,
        }
    }
}
