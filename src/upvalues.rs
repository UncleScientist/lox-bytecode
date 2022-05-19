use std::cell::RefCell;
use std::rc::Rc;

use crate::value::*;

#[derive(Debug)]
pub struct Upvalue {
    location: Rc<RefCell<Value>>,
}

impl Upvalue {
    pub fn new(value: &Rc<RefCell<Value>>) -> Self {
        Self {
            location: Rc::clone(value),
        }
    }

    pub fn value(&self) -> Rc<RefCell<Value>> {
        Rc::clone(&self.location)
    }

    pub fn set(&self, value: &Rc<RefCell<Value>>) {
        *self.location.borrow_mut() = value.borrow().clone()
    }
}
