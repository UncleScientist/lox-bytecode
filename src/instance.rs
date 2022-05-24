use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

use crate::class::*;
use crate::value::*;

#[derive(Debug)]
pub struct Instance {
    klass: Rc<Class>,
    fields: HashMap<String, Value>,
}

impl Instance {
    pub fn new(klass: Rc<Class>) -> Self {
        Self {
            klass: Rc::clone(&klass),
            fields: HashMap::new(),
        }
    }
}

impl Display for Instance {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{} instance", self.klass)
    }
}
