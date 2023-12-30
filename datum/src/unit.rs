use std::fmt::{Display, Formatter};

/// A `datum::unit::Unit` gives the unit associated with the `Value` stored in a `Datum`.
///
/// `Unit`s can be used to ensure that only sensible additions and aggregations of data are performed.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Unit {
    Unitless,
    PoweredOn,
    DegreesC,
}

/// Allows `Unit`s to be converted to `String`s with `to_string()`.
impl Display for Unit {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            Unit::Unitless => "",
            Unit::PoweredOn => "⏼",
            Unit::DegreesC => "°C",
        };

        write!(f, "{}", string)
    }
}

impl Unit {
    /// Attempts to parse a `Unit` from the provided string or string slice.
    pub fn parse<S: Into<String>>(s: S) -> Result<Unit, String> {
        let string = s.into();

        if string.is_empty() {
            Ok(Unit::Unitless)
        } else if string == "⏼" {
            Ok(Unit::PoweredOn)
        } else if string == "°C" {
            Ok(Unit::DegreesC)
        } else {
            Err(format!("cannot parse '{}' as a Unit", string))
        }
    }
}

#[cfg(test)]
mod datum_unit_tests {
    use super::*;

    #[test]
    fn test_display_and_parse_unitless() {
        let expected = Unit::Unitless;
        let serialized = expected.to_string();
        let actual = Unit::parse(serialized);
        assert_eq!(actual, Ok(expected))
    }

    #[test]
    fn test_display_and_parse_powered_on() {
        let expected = Unit::PoweredOn;
        let serialized = expected.to_string();
        let actual = Unit::parse(serialized);
        assert_eq!(actual, Ok(expected))
    }

    #[test]
    fn test_display_and_parse_degrees_c() {
        let expected = Unit::DegreesC;
        let serialized = expected.to_string();
        let actual = Unit::parse(serialized);
        assert_eq!(actual, Ok(expected))
    }

    #[test]
    fn test_parse_string() {
        let serialized = String::from("⏼");
        let actual = Unit::parse(serialized);
        assert_eq!(actual, Ok(Unit::PoweredOn))
    }

    #[test]
    fn test_parse_string_slice() {
        let serialized = "°C";
        let actual = Unit::parse(serialized);
        assert_eq!(actual, Ok(Unit::DegreesC))
    }

    #[test]
    fn test_parse_failure() {
        let serialized = "blorp";
        let actual = Unit::parse(serialized);
        let msg = String::from("cannot parse 'blorp' as a Unit");
        assert_eq!(actual, Err(msg))
    }
}
