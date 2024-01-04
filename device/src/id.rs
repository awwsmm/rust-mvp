use std::fmt::{Display, Formatter};

/// `Id` is a unique, immutable identifier associated with a `Device`.
#[derive(PartialEq, Debug, Clone, Eq, Hash)]
pub struct Id(String);

/// Allows `Id`s to be converted to `String`s with `to_string()`.
impl Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Id {
    pub fn new<S: Into<String>>(s: S) -> Id {
        Id(s.into())
    }
}

#[cfg(test)]
mod device_id_tests {
    use super::*;

    #[test]
    fn test_display() {
        let id = Id(String::from("id"));
        let actual = id.to_string();
        assert_eq!(actual, "id");
    }

    #[test]
    fn test_new() {
        let id = Id::new("id");
        let actual = id.to_string();
        assert_eq!(actual, "id");
    }
}
