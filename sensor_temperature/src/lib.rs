use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

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
    environment: Arc<Mutex<Option<ServiceInfo>>>,
    controller: Arc<Mutex<Option<ServiceInfo>>>,
    data: Arc<Mutex<VecDeque<Datum>>>,
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

    fn get_handler(&self) -> Handler {
        self.default_handler()
    }
}

impl Sensor for TemperatureSensor {
    fn new(id: Id, name: Name) -> TemperatureSensor {
        TemperatureSensor {
            id,
            name,
            environment: Arc::new(Mutex::new(None)),
            controller: Arc::new(Mutex::new(None)),
            data: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    fn get_environment(&self) -> &Arc<Mutex<Option<ServiceInfo>>> {
        &self.environment
    }

    fn get_controller(&self) -> &Arc<Mutex<Option<ServiceInfo>>> {
        &self.controller
    }

    fn get_datum_value_type(&self) -> Kind {
        Kind::Float
    }

    fn get_datum_unit(&self) -> Unit {
        Unit::DegreesC
    }

    fn get_data(&self) -> &Arc<Mutex<VecDeque<Datum>>> {
        &self.data
    }
}
