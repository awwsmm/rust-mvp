use datum::{Datum, DatumUnit};
use device::handler::Handler;
use device::id::Id;
use device::model::Model;
use device::name::Name;
use device::Device;
use sensor::Sensor;
use std::net::TcpStream;

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

    fn get_handler() -> Handler {
        let handler: for<'a> fn(&'a mut TcpStream) = |stream| Self::handle(stream, Self::get_datum);
        Handler::new(handler)
    }
}

impl Sensor for TemperatureSensor {
    fn get_datum() -> Datum {
        // TODO should query Environment
        Datum::new_now(25.0, DatumUnit::DegreesC)
    }
}

impl TemperatureSensor {
    pub fn new(id: Id, model: Model, name: Name) -> TemperatureSensor {
        TemperatureSensor { id, model, name }
    }
}
