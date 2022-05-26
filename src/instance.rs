use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

use crate::class::*;
use crate::value::*;

#[derive(Debug)]
pub struct Instance {
    klass: Rc<Class>,
    fields: RefCell<HashMap<String, Value>>,
}

impl Display for Instance {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{} instance", self.klass)
    }
}

impl Instance {
    pub fn new(klass: Rc<Class>) -> Self {
        Self {
            klass: Rc::clone(&klass),
            fields: RefCell::new(HashMap::new()),
        }
    }

    pub fn get_field<T: Into<String>>(&self, name: T) -> Option<Value> {
        self.fields.borrow().get(&name.into()).cloned()
    }

    pub fn set_field<T: Into<String>>(&self, name: T, value: &Value) {
        self.fields.borrow_mut().insert(name.into(), value.clone());
    }

    pub fn get_class(&self) -> Rc<Class> {
        Rc::clone(&self.klass)
    }
}
