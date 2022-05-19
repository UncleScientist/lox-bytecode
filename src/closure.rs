use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

use crate::chunk::*;
use crate::function::*;
use crate::upvalues::*;
use crate::value::*;

#[derive(Debug)]
pub struct Closure {
    function: Rc<Function>,
    upvalues: RefCell<Vec<Rc<Upvalue>>>,
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
            upvalues: RefCell::new(Vec::new()),
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

    pub fn push_upvalue(&self, value: &Rc<RefCell<Value>>) {
        self.upvalues
            .borrow_mut()
            .push(Rc::new(Upvalue::new(value)));
    }

    pub fn get_upvalue(&self, offset: usize) -> Rc<RefCell<Value>> {
        self.upvalues.borrow()[offset].value()
    }

    pub fn modify(&self, offset: usize, value: &Rc<RefCell<Value>>) {
        self.upvalues.borrow_mut()[offset].set(value)
    }
}
