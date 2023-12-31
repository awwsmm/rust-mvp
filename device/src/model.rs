use std::fmt::{Display, Formatter};

/// `Model` gives a unique identifier for each _kind_ of `Device`.
///
/// There can be multiple `Device`s of the same `Model` on the network. Each _individual_ `Device` has a unique [`Id`](crate::Id) and a user-defined [`Name`](crate::Name).
///
/// The `Controller` uses `Model`s to understand how to process `Sensor` `Datum`s into `Actuator` `Command`s.
///
/// The `Environment` uses `Model`s to understand how to mutate its state in response to `Actuator` `Command`s.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Model {
    Controller,
    Environment,
    Unsupported,
    Thermo5000,
    // TODO add more models here as they are supported
}

/// Allows `Model`s to be converted to `String`s with `to_string()`.
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

    /// Attempts to parse a `Model` from the provided string or string slice.
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

#[cfg(test)]
mod device_model_tests {
    use super::*;

    #[test]
    fn test_display_and_parse_controller() {
        let expected = Model::Controller;
        let serialized = expected.to_string();
        let actual = Model::parse(serialized);
        assert_eq!(actual, Ok(expected))
    }

    #[test]
    fn test_display_and_parse_environment() {
        let expected = Model::Environment;
        let serialized = expected.to_string();
        let actual = Model::parse(serialized);
        assert_eq!(actual, Ok(expected))
    }

    #[test]
    fn test_display_and_parse_unsupported() {
        let expected = Model::Unsupported;
        let serialized = expected.to_string();
        let actual = Model::parse(serialized);
        assert_eq!(actual, Ok(expected))
    }

    #[test]
    fn test_display_and_parse_thermo_5000() {
        let expected = Model::Thermo5000;
        let serialized = expected.to_string();
        let actual = Model::parse(serialized);
        assert_eq!(actual, Ok(expected))
    }

    #[test]
    fn test_parse_failure() {
        let serialized = "blorp";
        let actual = Model::parse(serialized);
        let msg = String::from("unknown Model 'blorp'");
        assert_eq!(Err(msg), actual)
    }
}