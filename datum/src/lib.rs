use std::fmt::{Display, Formatter};

use chrono::{DateTime, Utc};

/// A `Datum` is a singular data point; a single measurement / observation of some `Attribute`.
///
/// It contains a typed `value`, a `unit` associated with that `value`, and a `timestamp`.
///
/// Note that it is not generically-typed (no `T` parameter). Data is communicated across HTTP / TCP
/// and is consumed by a frontend HTML app, so we will lose type safety at those interfaces. Storing
/// these data points in `Datum` structs anticipates this complication and tries to tackle it head-on.
#[derive(PartialEq, Debug, Clone)]
pub struct Datum {
    pub value: DatumValue,
    pub unit: DatumUnit,
    pub timestamp: DateTime<Utc>,
}

impl Display for Datum {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}@{}@{}",
            self.value,
            self.unit,
            self.timestamp.to_rfc3339()
        )
    }
}

impl Datum {
    pub fn get_as_bool(&self) -> Option<bool> {
        match self.value {
            DatumValue::Bool(value) => Some(value),
            _ => None,
        }
    }

    pub fn get_as_float(&self) -> Option<f32> {
        match self.value {
            DatumValue::Float(value) => Some(value),
            _ => None,
        }
    }

    pub fn get_as_int(&self) -> Option<i32> {
        match self.value {
            DatumValue::Int(value) => Some(value),
            _ => None,
        }
    }

    // TODO add other 'get_as_x' methods here as necessary
}

impl From<bool> for DatumValue {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

impl From<f32> for DatumValue {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<i32> for DatumValue {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

#[derive(Debug)]
pub enum DatumValueType {
    Bool,
    Float,
    Int,
}

impl Display for DatumValueType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            DatumValueType::Bool => "bool",
            DatumValueType::Float => "float",
            DatumValueType::Int => "int",
        };
        write!(f, "{}", str)
    }
}

impl DatumValueType {
    pub fn parse(string: &str) -> Result<DatumValueType, String> {
        if string == "bool" {
            Ok(DatumValueType::Bool)
        } else if string == "float" {
            Ok(DatumValueType::Float)
        } else if string == "int" {
            Ok(DatumValueType::Int)
        } else {
            Err(format!("cannot parse DatumValueType from: {}", string))
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum DatumValue {
    Bool(bool),
    Float(f32),
    Int(i32),
}

impl Display for DatumValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            DatumValue::Bool(value) => value.to_string(),
            DatumValue::Float(value) => {
                let str = value.to_string();
                // force serialized floats to end with .0 to distinguish them from ints
                if str.contains('.') {
                    str
                } else {
                    format!("{}.0", str)
                }
            }
            DatumValue::Int(value) => value.to_string(),
        };

        write!(f, "{}", string)
    }
}

impl DatumValue {
    pub fn parse(string: String) -> Result<DatumValue, String> {
        if let Ok(value) = string.parse() {
            Ok(DatumValue::Bool(value))
        } else if let Ok(value) = string.parse() {
            Ok(DatumValue::Int(value))
        } else if let Ok(value) = string.parse() {
            Ok(DatumValue::Float(value))
        } else {
            Err(format!("cannot parse '{}' as a DatumValue", string))
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy, Default)]
pub enum DatumUnit {
    #[default]
    Unitless,
    PoweredOn,
    DegreesC,
}

impl Display for DatumUnit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            DatumUnit::Unitless => "",
            DatumUnit::PoweredOn => "⏼",
            DatumUnit::DegreesC => "°C",
        };

        write!(f, "{}", string)
    }
}

impl DatumUnit {
    pub fn parse(string: &str) -> Result<DatumUnit, String> {
        if string.is_empty() {
            Ok(DatumUnit::Unitless)
        } else if string == "⏼" {
            Ok(DatumUnit::PoweredOn)
        } else if string == "°C" {
            Ok(DatumUnit::DegreesC)
        } else {
            Err(format!("cannot parse '{}' as a DatumUnit", string))
        }
    }
}

impl Datum {
    pub fn new<T: Into<DatumValue>>(value: T, unit: DatumUnit, timestamp: DateTime<Utc>) -> Datum {
        Datum {
            value: value.into(),
            unit,
            timestamp,
        }
    }

    pub fn new_now<T: Into<DatumValue>>(value: T, unit: DatumUnit) -> Datum {
        Datum::new(value, unit, Utc::now())
    }

    // FIXME reduce nesting of match statements here
    pub fn parse(string: &str) -> Result<Datum, String> {
        let mut pieces = string.split('@');

        match (pieces.next(), pieces.next(), pieces.next()) {
            (Some(value), Some(unit), Some(timestamp)) => {
                match DatumValue::parse(value.to_string()) {
                    Ok(value) => match DatumUnit::parse(unit) {
                        Ok(unit) => match timestamp.parse::<DateTime<Utc>>() {
                            Ok(timestamp) => Ok(Datum::new(value, unit, timestamp)),
                            Err(msg) => Err(msg.to_string()),
                        },
                        Err(msg) => Err(msg),
                    },
                    Err(msg) => Err(msg),
                }
            }
            _ => Err(format!("unable to parse '{}' as a Datum", string)),
        }
    }
}

#[cfg(test)]
mod datum_tests {
    use super::*;

    fn create<T: Into<DatumValue>>(value: T) -> Datum {
        Datum::new(value, DatumUnit::Unitless, Utc::now())
    }

    #[test]
    fn test_create_datum_bool() {
        let datum = create(true);
        assert_eq!(datum.get_as_bool(), Some(true));
    }

    #[test]
    fn test_create_datum_bool_failure() {
        let datum = create(42.0);
        assert_eq!(datum.get_as_bool(), None);
    }

    #[test]
    fn test_create_datum_float() {
        let datum = create(42.0);
        assert_eq!(datum.get_as_float(), Some(42.0));
    }

    #[test]
    fn test_create_datum_float_failure() {
        let datum = create(true);
        assert_eq!(datum.get_as_float(), None);
    }

    #[test]
    fn test_create_datum_int() {
        let datum = create(19);
        assert_eq!(datum.get_as_int(), Some(19));
    }

    #[test]
    fn test_create_datum_int_failure() {
        let datum = create(true);
        assert_eq!(datum.get_as_int(), None);
    }

    #[test]
    fn test_datum_parse_int() {
        let now = Utc::now();
        let string = format!("12@@{}", now.to_rfc3339());

        let expected = Datum::new(12, DatumUnit::Unitless, now);
        let actual = Datum::parse(string.as_str());

        assert_eq!(actual, Ok(expected))
    }

    #[test]
    fn test_datum_parse_float() {
        let now = Utc::now();
        let string = format!("12.0@⏼@{}", now.to_rfc3339());

        let expected = Datum::new(12.0, DatumUnit::PoweredOn, now);
        let actual = Datum::parse(string.as_str());

        assert_eq!(actual, Ok(expected))
    }

    #[test]
    fn test_datum_parse_bool() {
        let now = Utc::now();
        let string = format!("false@°C@{}", now.to_rfc3339());

        let expected = Datum::new(false, DatumUnit::DegreesC, now);
        let actual = Datum::parse(string.as_str());

        assert_eq!(actual, Ok(expected))
    }
}
