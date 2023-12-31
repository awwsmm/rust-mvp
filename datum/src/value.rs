use std::fmt::{Display, Formatter};

/// A `datum::value::Value` gives the raw (numeric, boolean, or other) value stored in a `Datum`.
///
/// **Design Decision**: `Datums`s are purposefully not generic (no `<T>` parameter). Instead, the raw
/// value of a `Datum` is contained within a `Value`. Generic types are difficult to work with when
/// there is a need for heterogeneous collections of data. In this codebase, there are occasions
/// where we, for example, map `Device` IDs to the kind of data they produce or collect. Doing this
/// with generically-typed `Datum`s is much more cumbersome than just "hiding" the type information
/// inside of a `Value` and only re-typing the data on deserialization, comparison, etc.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Value {
    Bool(bool),
    Float(f32),
    Int(i32),
    // TODO add more data types here as they are supported
}

/// Allows `Value`s to be converted to `String`s with `to_string()`.
impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Value::Bool(value) => value.to_string(),
            Value::Float(value) => {
                let str = value.to_string();
                // force serialized floats to end with .0 to distinguish them from ints
                if str.contains('.') {
                    str
                } else {
                    format!("{}.0", str)
                }
            }
            Value::Int(value) => value.to_string(),
        };

        write!(f, "{}", string)
    }
}

impl Value {
    /// Attempts to parse a `Value` from the provided string or string slice.
    pub fn parse<S: Into<String>>(s: S) -> Result<Value, String> {
        let string = s.into();

        if let Ok(value) = string.parse() {
            Ok(Value::Bool(value))
        } else if let Ok(value) = string.parse() {
            Ok(Value::Int(value))
        } else if let Ok(value) = string.parse() {
            Ok(Value::Float(value))
        } else {
            Err(format!("cannot parse '{}' as a Value", string))
        }
    }
}

/// Allows a `bool` to be automatically converted into a `Value::Bool`.
impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Self::Bool(value)
    }
}

/// Allows an `f32` to be automatically converted into a `Value::Float`.
impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

/// Allows an `i32` to be automatically converted into a `Value::Int`.
impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::Int(value)
    }
}

#[cfg(test)]
mod datum_value_tests {
    use super::*;

    #[test]
    fn test_display_and_parse_bool() {
        let expected = Value::Bool(true);
        let serialized = expected.to_string();
        let actual = Value::parse(serialized);

        assert_eq!(actual, Ok(expected))
    }

    #[test]
    fn test_display_and_parse_float_with_unnecessary_decimal_point() {
        let expected = Value::Float(42.0);
        let serialized = expected.to_string();
        let actual = Value::parse(serialized);

        assert_eq!(actual, Ok(expected))
    }

    #[test]
    fn test_display_and_parse_float_with_necessary_decimal_point() {
        let expected = Value::Float(42.1);
        let serialized = expected.to_string();
        let actual = Value::parse(serialized);

        assert_eq!(actual, Ok(expected))
    }

    #[test]
    fn test_display_and_parse_float_without_decimal_point() {
        let expected = Value::Float(42f32);
        let serialized = expected.to_string();
        let actual = Value::parse(serialized);

        assert_eq!(actual, Ok(expected))
    }

    #[test]
    fn test_display_and_parse_int() {
        let expected = Value::Int(42);
        let serialized = expected.to_string();
        let actual = Value::parse(serialized);

        assert_eq!(actual, Ok(expected))
    }

    #[test]
    fn test_parse_failure() {
        let serialized = "blorp";
        let actual = Value::parse(serialized);
        let msg = String::from("cannot parse 'blorp' as a Value");
        assert_eq!(actual, Err(msg))
    }

    #[test]
    fn test_value_from_bool() {
        let raw = false;
        let value: Value = raw.into();
        assert_eq!(Value::Bool(false), value)
    }

    #[test]
    fn test_value_from_float() {
        let raw = 42.0;
        let value: Value = raw.into();
        assert_eq!(Value::Float(42.0), value)
    }

    #[test]
    fn test_value_from_int() {
        let raw = 42;
        let value: Value = raw.into();
        assert_eq!(Value::Int(42), value)
    }
}
