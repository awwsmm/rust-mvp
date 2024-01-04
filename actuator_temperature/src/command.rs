use std::fmt::{Display, Formatter};

#[derive(PartialEq, Debug)]
pub enum Command {
    CoolBy(f32), // the Controller tells the Actuator to cool the Environment by 'x' degrees C
    HeatBy(f32), // the Controller tells the Actuator to heat the Environment by 'x' degrees C
}

impl actuator::Command for Command {}

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
    pub fn parse<S: Into<String>>(s: S) -> Result<Command, String> {
        let original = s.into();
        let mut string = original.clone();
        string.retain(|c| !c.is_whitespace());
        let string = string.trim_start_matches('{').trim_end_matches('}');
        let mut pieces = string.split(',');

        let name = pieces.next().unwrap().trim_start_matches(r#""name":""#).trim_end_matches('"');
        let value = pieces.next().unwrap().trim_start_matches(r#""value":""#).trim_end_matches('"');

        match (name, value) {
            ("CoolBy", value) => match value.parse() {
                Ok(temp) => Ok(Command::CoolBy(temp)),
                Err(_) => Err(format!("cannot parse {} as f32", value)),
            },
            ("HeatBy", value) => match value.parse() {
                Ok(temp) => Ok(Command::HeatBy(temp)),
                Err(_) => Err(format!("cannot parse {} as f32", value)),
            },
            _ => Err(format!("cannot parse {} as Command", string)),
        }
    }
}

#[cfg(test)]
mod actuator_temperature_command_tests {
    use super::*;

    fn serde(command: &Command) -> Result<Command, String> {
        let serialized = command.to_string();
        Command::parse(serialized.as_str())
    }

    #[test]
    fn test_serde_cool_to() {
        let command = Command::CoolBy(42.0);
        let deserialized = serde(&command);

        assert_eq!(deserialized, Ok(command))
    }

    #[test]
    fn test_serde_heat_to() {
        let command = Command::HeatBy(19.3);
        let deserialized = serde(&command);

        assert_eq!(deserialized, Ok(command))
    }
}
