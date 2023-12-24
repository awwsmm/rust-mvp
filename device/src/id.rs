use std::fmt::{Display, Formatter};

#[derive(PartialEq, Debug, Eq, Hash, Clone)]
pub struct Id(pub String);

impl Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Id {
    pub fn new(id: &str) -> Id {
        Id(String::from(id))
    }
}
