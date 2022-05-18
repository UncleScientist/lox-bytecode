use std::rc::Rc;

use crate::value::*;

#[derive(Debug)]
pub struct Upvalue {
    location: Rc<Value>,
}

impl Upvalue {
    pub fn new(value: &Rc<Value>) -> Self {
        Self {
            location: Rc::clone(value),
        }
    }

    pub fn value(&self) -> Rc<Value> {
        Rc::clone(&self.location)
    }
}
