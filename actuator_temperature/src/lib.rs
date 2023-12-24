use actuator::Actuator;
use device::{Device, Id, Model, Name};

use crate::command::Command;

mod command;

pub struct TemperatureActuator {
    id: Id,
    model: Model,
    name: Name,
}

impl Device for TemperatureActuator {
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

impl Actuator for TemperatureActuator {
    fn act(&self, _id: Id, command: String) {
        let command_is_valid = Command::parse(command.as_str()).is_ok();

        if command_is_valid {
            todo!() // send to Environment
        }
    }
}

impl TemperatureActuator {
    pub fn new(id: Id, model: Model, name: Name) -> TemperatureActuator {
        TemperatureActuator { id, model, name }
    }
}
