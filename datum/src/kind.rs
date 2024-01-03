use std::fmt::{Display, Formatter};

/// A `datum::kind::Kind` gives the type of the `Value` stored in a `Datum`.
///
/// It is useful for deserializing serialized `Datum`s.
#[derive(Debug, PartialEq)]
pub enum Kind {
    Bool,
    Float,
    Int,
    // add more data types here as they are supported
}

/// Allows `Kind`s to be converted to `String`s with `to_string()`.
impl Display for Kind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            Kind::Bool => "bool",
            Kind::Float => "float",
            Kind::Int => "int",
        };
        write!(f, "{}", str)
    }
}

impl Kind {
    /// Attempts to parse a `Kind` from the provided string or string slice.
    pub fn parse<S: Into<String>>(s: S) -> Result<Kind, String> {
        let string = s.into();

        if string == "bool" {
            Ok(Kind::Bool)
        } else if string == "float" {
            Ok(Kind::Float)
        } else if string == "int" {
            Ok(Kind::Int)
        } else {
            Err(format!("cannot parse DatumValueType from: {}", string))
        }
    }
}

#[cfg(test)]
mod datum_kind_tests {
    use super::*;

    #[test]
    fn test_display_and_parse_bool() {
        let expected = Kind::Bool;
        let serialized = expected.to_string();
        let actual = Kind::parse(serialized);
        assert_eq!(actual, Ok(expected))
    }

    #[test]
    fn test_display_and_parse_float() {
        let expected = Kind::Float;
        let serialized = expected.to_string();
        let actual = Kind::parse(serialized);
        assert_eq!(actual, Ok(expected))
    }

    #[test]
    fn test_display_and_parse_int() {
        let expected = Kind::Int;
        let serialized = expected.to_string();
        let actual = Kind::parse(serialized);
        assert_eq!(actual, Ok(expected))
    }

    #[test]
    fn test_parse_string() {
        let serialized = String::from("float");
        let kind = Kind::parse(serialized);
        assert_eq!(Ok(Kind::Float), kind)
    }

    #[test]
    fn test_parse_string_slice() {
        let serialized = "int";
        let kind = Kind::parse(serialized);
        assert_eq!(Ok(Kind::Int), kind)
    }

    #[test]
    fn test_parse_failure() {
        let serialized = "blorp";
        let actual = Kind::parse(serialized);
        let msg = String::from("cannot parse DatumValueType from: blorp");
        assert_eq!(Err(msg), actual)
    }
}
