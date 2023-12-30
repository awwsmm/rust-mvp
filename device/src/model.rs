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
        let str = match self {
            Model::Controller => "controller",
            Model::Environment => "environment",
            Model::Unsupported => "unsupported",
            Model::Thermo5000 => "thermo5000",
        };

        write!(f, "{}", str)
    }
}

impl Model {
    pub fn parse<S: Into<String>>(s: S) -> Result<Model, String> {
        let string = s.into();
        match string.as_str() {
            "controller" => Ok(Model::Controller),
            "environment" => Ok(Model::Environment),
            "unsupported" => Ok(Model::Unsupported),
            "thermo5000" => Ok(Model::Thermo5000),
            _ => Err(format!("unknown Model '{}'", string)),
        }
    }
}
//
// #[cfg(test)]
// mod device_id_tests {
//     use super::*;
//
//     #[test]
//     fn test_display() {
//
//     }
//
// }
