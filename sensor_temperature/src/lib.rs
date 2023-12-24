use datum::{Datum, DatumUnit};
use device::id::Id;
use device::model::Model;
use device::name::Name;
use device::Device;
use sensor::Sensor;

pub struct TemperatureSensor {
    id: Id,
    model: Model,
    name: Name,
}

impl Device for TemperatureSensor {
    fn get_name(&self) -> &Name {
        &self.name
    }

    fn get_model(&self) -> &Model {
        &self.model
    }

    fn get_id(&self) -> &Id {
        &self.id
    }
}

impl Sensor for TemperatureSensor {
    fn get_datum(&self) -> Datum {
        // TODO should query Environment
        Datum::new_now(25.0, DatumUnit::DegreesC)
    }
}

impl TemperatureSensor {
    pub fn new(id: Id, model: Model, name: Name) -> TemperatureSensor {
        TemperatureSensor { id, model, name }
    }
}
