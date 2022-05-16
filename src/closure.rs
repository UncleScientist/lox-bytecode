use std::fmt::Display;
use std::rc::Rc;

use crate::chunk::*;
use crate::function::*;

#[derive(Debug)]
pub struct Closure {
    function: Rc<Function>,
    // <- captured variables go here
}

impl Display for Closure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        self.function.fmt(f)
    }
}

impl Closure {
    pub fn new(function: Rc<Function>) -> Self {
        Self {
            function: Rc::clone(&function),
        }
    }

    pub fn arity(&self) -> usize {
        self.function.arity()
    }

    pub fn get_chunk(&self) -> Rc<Chunk> {
        self.function.get_chunk()
    }

    pub fn stack_name(&self) -> &str {
        self.function.stack_name()
    }
}
