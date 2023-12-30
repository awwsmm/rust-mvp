#[derive(Clone, Copy)]
pub enum Model {
    Controller,
    Environment,
    Unsupported,
    Thermo5000,
}

impl Model {
    pub fn parse(string: &str) -> Result<Model, String> {
        match string {
            "controller" => Ok(Model::Controller),
            "environment" => Ok(Model::Environment),
            "unsupported" => Ok(Model::Unsupported),
            "thermo5000" => Ok(Model::Thermo5000),
            _ => Err(format!("unknown Model '{}'", string)),
        }
    }

    /// Returns an mDNS-fullname-safe id for this `Model`.
    pub fn id(&self) -> String {
        match self {
            Model::Controller => "controller",
            Model::Environment => "environment",
            Model::Unsupported => "unsupported",
            Model::Thermo5000 => "thermo5000",
        }
        .into()
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
