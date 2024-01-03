use std::fmt::{Display, Formatter};

use chrono::{DateTime, Utc};

use crate::unit::Unit;
use crate::value::Value;

pub mod kind;
pub mod unit;
pub mod value;

/// A `Datum` is a singular data point, a single measurement / observation of some attribute of the environment.
///
/// It contains a typed `value`, a `unit` associated with that value, and a `timestamp`.
///
/// **Design Decision**: `Datum`s are purposefully not generically-typed (no `T` parameter). Data is
/// communicated across HTTP / TCP and is consumed by a front-end HTML app, so we will lose type
/// safety at those interfaces. Storing these data points in `Datum` structs anticipates this
/// complication and tries to tackle it head-on.
///
/// **Design Decision**: `timestamp`s are of type `DateTime<Utc>` because the external crate `chrono`
/// provides useful methods for converting `DateTime<Utc>` values to strings / parsing them from
/// strings. In this codebase, `timestamp`s are serialized to / deserialized from
/// [RFC 3339](https://datatracker.ietf.org/doc/html/rfc3339) /
/// [ISO 8601](https://en.wikipedia.org/wiki/ISO_8601)-formatted strings. This external dependency
/// could be removed if timestamp de/serialization were implemented here.
#[derive(PartialEq, Debug, Clone)]
pub struct Datum {
    pub value: Value,
    pub unit: Unit,
    pub timestamp: DateTime<Utc>,
}

/// Allows `Datum`s to be converted to `String`s with `to_string()`.
impl Display for Datum {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            r#"{{"value":"{}","unit":"{}","timestamp":"{}"}}"#,
            self.value,
            self.unit,
            self.timestamp.to_rfc3339()
        )
    }
}

impl Datum {
    pub fn new<T: Into<Value>>(value: T, unit: Unit, timestamp: DateTime<Utc>) -> Datum {
        Datum {
            value: value.into(),
            unit,
            timestamp,
        }
    }

    /// Creates a `new` `Datum` with the `timestamp` set to `Utc::now()`.
    pub fn new_now<T: Into<Value>>(value: T, unit: Unit) -> Datum {
        Datum::new(value, unit, Utc::now())
    }

    /// Attempts to parse a `Datum` from the provided string or string slice.
    pub fn parse<S: Into<String>>(s: S) -> Result<Datum, String> {
        let original = s.into();
        let mut string = original.clone();
        string.retain(|c| !c.is_whitespace());
        let string = string.trim_start_matches('{').trim_end_matches('}');
        let mut pieces = string.split(',');

        match (pieces.next(), pieces.next(), pieces.next()) {
            (Some(value), Some(unit), Some(timestamp)) => match (
                Value::parse(
                    value
                        .trim_start_matches(r#""value":""#)
                        .trim_end_matches('"'),
                ),
                Unit::parse(unit.trim_start_matches(r#""unit":""#).trim_end_matches('"')),
                timestamp
                    .trim_start_matches(r#""timestamp":""#)
                    .trim_end_matches('"')
                    .parse::<DateTime<Utc>>(),
            ) {
                (Ok(value), Ok(unit), Ok(timestamp)) => Ok(Datum::new(value, unit, timestamp)),
                (Err(msg), _, _) => Err(msg),
                (_, Err(msg), _) => Err(msg),
                (_, _, Err(msg)) => Err(msg.to_string()),
            },
            _ => Err(format!(
                "'{}' is not formatted like a serialized Datum",
                original
            )),
        }
    }

    /// Attempts to convert this `Datum` into a raw `bool` value.
    pub fn get_as_bool(&self) -> Option<bool> {
        match self.value {
            Value::Bool(value) => Some(value),
            _ => None,
        }
    }

    /// Attempts to convert this `Datum` into a raw `float` value.
    pub fn get_as_float(&self) -> Option<f32> {
        match self.value {
            Value::Float(value) => Some(value),
            _ => None,
        }
    }

    /// Attempts to convert this `Datum` into a raw `int` value.
    pub fn get_as_int(&self) -> Option<i32> {
        match self.value {
            Value::Int(value) => Some(value),
            _ => None,
        }
    }
}

#[cfg(test)]
mod datum_tests {
    use std::time::Duration;

    use super::*;

    fn create<T: Into<Value>>(value: T) -> Datum {
        Datum::new(value, Unit::Unitless, Utc::now())
    }

    #[test]
    fn test_create_datum_get_as_bool() {
        let datum = create(true);
        assert_eq!(datum.get_as_bool(), Some(true));
    }

    #[test]
    fn test_create_datum_get_as_bool_failure() {
        let datum = create(42.0);
        assert_eq!(datum.get_as_bool(), None);
    }

    #[test]
    fn test_create_datum_get_as_float() {
        let datum = create(42.0);
        assert_eq!(datum.get_as_float(), Some(42.0));
    }

    #[test]
    fn test_create_datum_get_as_float_failure() {
        let datum = create(true);
        assert_eq!(datum.get_as_float(), None);
    }

    #[test]
    fn test_create_datum_get_as_int() {
        let datum = create(19);
        assert_eq!(datum.get_as_int(), Some(19));
    }

    #[test]
    fn test_create_datum_get_as_int_failure() {
        let datum = create(true);
        assert_eq!(datum.get_as_int(), None);
    }

    #[test]
    fn test_datum_parse_int() {
        let now = Utc::now();
        let expected = Datum::new(12, Unit::Unitless, now);
        let serialized = expected.to_string();
        let actual = Datum::parse(serialized);
        assert_eq!(actual, Ok(expected))
    }

    #[test]
    fn test_datum_parse_float() {
        let now = Utc::now();
        let expected = Datum::new(12.0, Unit::Unitless, now);
        let serialized = expected.to_string();
        let actual = Datum::parse(serialized);
        assert_eq!(actual, Ok(expected))
    }

    #[test]
    fn test_datum_parse_bool() {
        let now = Utc::now();
        let expected = Datum::new(false, Unit::Unitless, now);
        let serialized = expected.to_string();
        let actual = Datum::parse(serialized);
        assert_eq!(actual, Ok(expected))
    }

    #[test]
    fn test_create_new_now() {
        let earlier = Utc::now();
        std::thread::sleep(Duration::from_millis(1));
        let datum = Datum::new_now(42.0, Unit::DegreesC);
        std::thread::sleep(Duration::from_millis(1));
        let later = Utc::now();

        assert!(datum.timestamp > earlier);
        assert!(datum.timestamp < later);
    }

    #[test]
    fn test_parse_failure_not_enough_pieces() {
        //                     r#"{"value":"42.0","unit":"°C","timestamp":"2024-01-03T18:03:21.742821+00:00"}"#
        let serialized = r#"{"value":"42.0","unit":"°C"}"#;
        let actual = Datum::parse(serialized);
        let msg = format!("'{}' is not formatted like a serialized Datum", serialized);

        assert_eq!(actual, Err(msg))
    }

    #[test]
    fn test_parse_failure_bad_value() {
        //                     r#"{"value":"42.0","unit":"°C","timestamp":"2024-01-03T18:03:21.742821+00:00"}"#
        let serialized =
            r#"{"value":"42P0","unit":"°C","timestamp":"2024-01-03T18:03:21.742821+00:00"}"#;
        let actual = Datum::parse(serialized);
        let msg = "cannot parse '42P0' as a Value".to_string();

        assert_eq!(actual, Err(msg))
    }

    #[test]
    fn test_parse_failure_bad_unit() {
        //                     r#"{"value":"42.0","unit":"°C","timestamp":"2024-01-03T18:03:21.742821+00:00"}"#
        let serialized =
            r#"{"value":"42.0","unit":"oC","timestamp":"2024-01-03T18:03:21.742821+00:00"}"#;
        let actual = Datum::parse(serialized);
        let msg = "cannot parse 'oC' as a Unit".to_string();

        assert_eq!(actual, Err(msg))
    }

    #[test]
    fn test_parse_failure_bad_timestamp() {
        //                     r#"{"value":"42.0","unit":"°C","timestamp":"2024-01-03T18:03:21.742821+00:00"}"#
        let serialized =
            r#"{"value":"42.0","unit":"°C","timestamp":"2_24-01-03T18:03:21.742821+00:00"}"#;
        let actual = Datum::parse(serialized);
        let msg = "input contains invalid characters".to_string();

        assert_eq!(actual, Err(msg))
    }
}
