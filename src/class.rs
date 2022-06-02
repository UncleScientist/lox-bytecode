use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::rc::Rc;

use crate::closure::*;
use crate::value::*;

#[derive(Debug)]
pub struct Class {
    name: String,
    methods: RefCell<HashMap<String, Rc<Closure>>>,
    init: RefCell<Option<Rc<Closure>>>,
}

impl Class {
    pub fn new(name: String) -> Self {
        Self {
            name,
            methods: RefCell::new(HashMap::new()),
            init: RefCell::new(None),
        }
    }

    pub fn set_init_method(&self, closure: Rc<Closure>) {
        self.init.replace(Some(closure));
    }

    pub fn get_init_method(&self) -> Option<Rc<Closure>> {
        if self.init.borrow().is_none() {
            None
        } else {
            Some(self.init.borrow().as_ref().unwrap().clone())
        }
    }

    pub fn add_method(&self, name: &str, value: &Value) {
        if let Value::Closure(closure) = value {
            self.methods
                .borrow_mut()
                .insert(name.to_string(), closure.clone());
        }
    }

    pub fn get_method(&self, name: &str) -> Option<Rc<Closure>> {
        self.methods.borrow().get(name).cloned()
    }
}

impl Display for Class {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}", self.name)
    }
}
