use actuator::Actuator;
use device::handler::Handler;
use device::id::Id;
use device::model::Model;
use device::name::Name;
use device::Device;

pub mod command;

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

    fn get_handler(&self) -> Handler {
        Self::default_handler()
    }
}

impl Actuator for TemperatureActuator {
    // fn act(&self, _id: Id, command: String) {
    //     let command_is_valid = Command::parse(command.as_str()).is_ok();
    //
    //     if command_is_valid {
    //         todo!() // send to Environment
    //     }
    // }
}

impl TemperatureActuator {
    pub fn new(id: Id, model: Model, name: Name) -> TemperatureActuator {
        TemperatureActuator { id, model, name }
    }
}
