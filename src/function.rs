use crate::chunk::*;
use std::fmt::Display;

use std::cell::RefCell;

pub struct Function {
    arity: usize,
    chunk: RefCell<Chunk>,
    name: String,
}

impl PartialOrd for Function {
    fn partial_cmp(&self, _: &Self) -> Option<std::cmp::Ordering> {
        panic!("comparing the ord of two functions");
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

impl Clone for Function {
    fn clone(&self) -> Self {
        Function {
            arity: self.arity,
            chunk: self.chunk.clone(),
            name: self.name.clone(),
        }
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "<fn {}>", self.name)
    }
}

impl Function {
    pub fn new() -> Self {
        Function {
            arity: 0,
            chunk: RefCell::new(Chunk::new()),
            name: "".to_string(),
        }
    }
}
