use std::fmt::{Display, Formatter};

/// These are the `Command`s provided by the `TemperatureActuator`.
#[derive(PartialEq, Debug)]
pub enum Command {
    CoolBy(f32), // the Controller tells the Actuator to cool the Environment by 'x' degrees C
    HeatBy(f32), // the Controller tells the Actuator to heat the Environment by 'x' degrees C
}

impl actuator::Command for Command {}

/// Allows `Command`s to be converted to `String`s with `to_string()`.
impl Display for Command {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let (name, value) = match self {
            Command::CoolBy(temp) => ("CoolBy", temp),
            Command::HeatBy(temp) => ("HeatBy", temp),
        };

        write!(f, r#"{{"name":"{}","value":"{}"}}"#, name, value)
    }
}

impl Command {
    /// Attempts to parse a `Command` from the provided string or string slice.
    pub fn parse<S: Into<String>>(s: S) -> Result<Command, String> {
        let original = s.into();
        let mut string = original.clone();
        string.retain(|c| !c.is_whitespace());
        let string = string.trim_start_matches('{').trim_end_matches('}');
        let mut pieces = string.split(',');

        match (pieces.next(), pieces.next()) {
            (Some(name), Some(command)) => {
                let name = name.trim_start_matches(r#""name":""#).trim_end_matches('"');
                let value = command.trim_start_matches(r#""value":""#).trim_end_matches('"');

                match (name, value) {
                    ("CoolBy", value) => match value.parse() {
                        Ok(temp) => Ok(Command::CoolBy(temp)),
                        Err(_) => Err(format!("cannot parse '{}' as f32", value)),
                    },
                    ("HeatBy", value) => match value.parse() {
                        Ok(temp) => Ok(Command::HeatBy(temp)),
                        Err(_) => Err(format!("cannot parse '{}' as f32", value)),
                    },
                    _ => Err(format!("cannot parse '{}' as Command", original)),
                }
            }
            _ => Err(format!("cannot parse '{}' as Command", original)),
        }
    }
}

#[cfg(test)]
mod actuator_temperature_command_tests {
    use super::*;

    fn serde(command: &Command) -> Result<Command, String> {
        let serialized = command.to_string();

        println!("{}", serialized);

        Command::parse(serialized.as_str())
    }

    #[test]
    fn test_serde_cool_by() {
        let command = Command::CoolBy(42.0);
        let deserialized = serde(&command);

        assert_eq!(deserialized, Ok(command))
    }

    #[test]
    fn test_serde_heat_by() {
        let command = Command::HeatBy(19.3);
        let deserialized = serde(&command);

        assert_eq!(deserialized, Ok(command))
    }

    #[test]
    fn test_parse_failure_cool_by() {
        let serialized = r#"{"name":"CoolBy","value":":("}"#;
        let actual = Command::parse(serialized);
        assert_eq!(actual, Err("cannot parse ':(' as f32".to_string()))
    }

    #[test]
    fn test_parse_failure_heat_by() {
        let serialized = r#"{"name":"HeatBy","value":":("}"#;
        let actual = Command::parse(serialized);
        assert_eq!(actual, Err("cannot parse ':(' as f32".to_string()))
    }

    #[test]
    fn test_parse_failure() {
        let serialized = r#"not a command"#;
        let actual = Command::parse(serialized);
        assert_eq!(actual, Err(format!("cannot parse '{}' as Command", serialized)))
    }

    #[test]
    fn test_parse_failure_bad_value() {
        let serialized = r#"{"name":"Blorp","value":":("}"#;
        let actual = Command::parse(serialized);
        assert_eq!(actual, Err(format!("cannot parse '{}' as Command", serialized)))
    }
}
