use std::fmt::{Display, Formatter};

#[derive(PartialEq, Debug, Clone)]
pub struct Name(pub String);

impl Display for Name {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Name {
    pub fn new(name: &str) -> Name {
        Name(String::from(name))
    }
}
