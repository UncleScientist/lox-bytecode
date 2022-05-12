use crate::chunk::*;
use std::fmt::Display;

use std::rc::Rc;

#[derive(Debug, Default)]
pub struct Function {
    arity: usize,
    pub chunk: Rc<Chunk>,
    name: String,
}

impl PartialOrd for Function {
    fn partial_cmp(&self, _: &Self) -> Option<std::cmp::Ordering> {
        panic!("comparing the ord of two functions");
    }
}

impl PartialEq for Function {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
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
        if self.name.is_empty() {
            write!(f, "<script>")
        } else {
            write!(f, "<fn {}>", self.name)
        }
    }
}

impl Function {
    pub fn new<T: Into<String>>(arity: usize, chunk: &Rc<Chunk>, name: T) -> Self {
        Self {
            arity,
            chunk: Rc::clone(chunk),
            name: name.into(),
        }
    }

    pub fn toplevel(chunk: &Rc<Chunk>) -> Self {
        Self {
            arity: 0,
            chunk: Rc::clone(chunk),
            name: "".to_string(),
        }
    }

    pub fn get_chunk(&self) -> Rc<Chunk> {
        Rc::clone(&self.chunk)
    }

    pub fn arity(&self) -> usize {
        self.arity
    }

    pub fn stack_name(&self) -> &str {
        if self.name.is_empty() {
            "script"
        } else {
            self.name.as_str()
        }
    }
}
