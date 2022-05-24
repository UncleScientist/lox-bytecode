use std::fmt::{Debug, Display, Formatter};

#[derive(Debug)]
pub struct Class {
    name: String,
}

impl Display for Class {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.name)
    }
}
