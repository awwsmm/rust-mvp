use std::fmt::{Display, Formatter};

#[derive(Clone, Copy)]
pub enum Model {
    Controller,
    Environment,
    Unsupported,
    Thermo5000,
}

impl Display for Model {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Model::Controller => "CONTROLLER",
            Model::Environment => "ENVIRONMENT",
            Model::Unsupported => "UNSUPPORTED",
            Model::Thermo5000 => "Thermo-5000",
        };

        write!(f, "{}", string)
    }
}

impl Model {
    pub fn parse(string: &str) -> Result<Model, String> {
        match string {
            "CONTROLLER" => Ok(Model::Controller),
            "ENVIRONMENT" => Ok(Model::Environment),
            "UNSUPPORTED" => Ok(Model::Unsupported),
            "Thermo-5000" => Ok(Model::Thermo5000),
            _ => Err(format!("unknown Model '{}'", string)),
        }
    }
}
