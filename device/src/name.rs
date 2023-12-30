use std::fmt::{Display, Formatter};

#[derive(PartialEq, Debug, Clone)]
pub struct Name(String);

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Name {
    pub fn new<S: Into<String>>(s: S) -> Name {
        Name(s.into())
    }
}

#[cfg(test)]
mod device_name_tests {
    use super::*;

    #[test]
    fn test_display() {
        let name = Name(String::from("name"));
        let actual = name.to_string();
        assert_eq!(actual, "name");
    }

    #[test]
    fn test_new() {
        let name = Name::new("name");
        let actual = name.to_string();
        assert_eq!(actual, "name");
    }
}
