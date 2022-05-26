use std::fmt::Display;
use std::rc::Rc;

use crate::closure::*;
use crate::value::*;

#[derive(Debug)]
pub struct BoundMethod {
    #[allow(dead_code)]
    receiver: Value,
    method: Rc<Closure>,
}

impl BoundMethod {
    pub fn new(receiver: &Value, method: &Rc<Closure>) -> Self {
        Self {
            receiver: receiver.clone(),
            method: Rc::clone(method),
        }
    }

    pub fn get_closure(&self) -> Rc<Closure> {
        Rc::clone(&self.method)
    }
}

impl Display for BoundMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.method)
    }
}
