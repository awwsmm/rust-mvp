use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use mdns_sd::{ServiceDaemon, ServiceInfo};

use datum::{Datum, DatumUnit};
use device::handler::Handler;
use device::id::Id;
use device::model::Model;
use device::name::Name;
use device::Device;
use sensor::Sensor;

pub struct TemperatureSensor {
    id: Id,
    name: Name,
    pub env: Arc<Mutex<HashMap<Id, ServiceInfo>>>,
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

    fn get_handler(&self) -> Handler {
        Self::default_handler()
    }

    fn start(ip: IpAddr, port: u16, id: Id, name: Name, mdns: Arc<ServiceDaemon>) {
        let device = Self::new(id, name);

        let mut targets = HashMap::new();
        targets.insert("_controller".into(), &device.env);

        device.run(ip, port, "_sensor", targets, mdns);
    }
}

impl Sensor for TemperatureSensor {
    fn get_datum() -> Datum {
        // TODO should query Environment
        Datum::new_now(25.0, DatumUnit::DegreesC)
    }
}

impl TemperatureSensor {
    pub fn new(id: Id, name: Name) -> TemperatureSensor {
        TemperatureSensor {
            id,
            name,
            env: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn start_new(
        ip: IpAddr,
        port: u16,
        id: Id,
        name: Name,
        mdns: Arc<ServiceDaemon>,
    ) -> JoinHandle<()> {
        std::thread::spawn(move || {
            println!(">>> [sensor_temp start_new] SPAWNED A NEW THREAD");

            let device = Self::new(id, name);

            let mut targets = HashMap::new();
            targets.insert("_controller".into(), &device.env);

            device.run_new(ip, port, "_sensor", targets, mdns);
        })
    }
}
