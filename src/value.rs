use std::any::Any;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, Div, Mul, Neg, Sub};
use std::rc::Rc;

use crate::closure::*;
use crate::function::*;

pub trait NativeFunc {
    fn call(&self, arg_count: usize, args: &[Rc<Value>]) -> Value;
}

impl Debug for dyn NativeFunc {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "<native fn>")
    }
}

#[derive(Debug)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    Nil,
    Str(String),
    Func(Rc<Function>),
    Native(Rc<dyn NativeFunc>),
    Closure(Rc<Closure>),
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (Value::Boolean(a), Value::Boolean(b)) => a.partial_cmp(b),
            (Value::Number(a), Value::Number(b)) => a.partial_cmp(b),
            (Value::Str(a), Value::Str(b)) => a.partial_cmp(b),
            _ => None,
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Boolean(a), Value::Boolean(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::Str(a), Value::Str(b)) => a.cmp(b) == Ordering::Equal,
            (Value::Nil, Value::Nil) => true,
            (Value::Func(a), Value::Func(b)) => Rc::ptr_eq(a, b),
            (Value::Native(a), Value::Native(b)) => a.type_id() == b.type_id(),
            _ => false,
        }
    }
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Boolean(b) => Value::Boolean(*b),
            Value::Number(n) => Value::Number(*n),
            Value::Nil => Value::Nil,
            Value::Str(s) => Value::Str(s.clone()),
            Value::Func(f) => Value::Func(Rc::clone(f)),
            Value::Native(n) => Value::Native(Rc::clone(n)),
            Value::Closure(c) => Value::Closure(Rc::clone(c)),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Value::Boolean(b) => write!(f, "{b}"),
            Value::Number(n) => write!(f, "{n}"),
            Value::Nil => write!(f, "nil"),
            Value::Str(s) => write!(f, "{s}"),
            Value::Func(func) => write!(f, "{func}"),
            Value::Native(_) => write!(f, "<native fn>"),
            Value::Closure(closure) => write!(f, "{closure}"),
        }
    }
}

impl Add for &Value {
    type Output = Value;

    fn add(self, other: &Value) -> Value {
        match (self, other) {
            (&Value::Number(a), &Value::Number(b)) => Value::Number(a + b),
            _ => panic!("Invalid operation"),
        }
    }
}

impl Sub for &Value {
    type Output = Value;

    fn sub(self, other: &Value) -> Value {
        match (self, other) {
            (&Value::Number(a), &Value::Number(b)) => Value::Number(a - b),
            _ => panic!("Invalid operation"),
        }
    }
}

impl Mul for &Value {
    type Output = Value;

    fn mul(self, other: &Value) -> Value {
        match (self, other) {
            (&Value::Number(a), &Value::Number(b)) => Value::Number(a * b),
            _ => panic!("Invalid operation"),
        }
    }
}

impl Div for &Value {
    type Output = Value;

    fn div(self, other: &Value) -> Value {
        match (self, other) {
            (&Value::Number(a), &Value::Number(b)) => Value::Number(a / b),
            _ => panic!("Invalid operation"),
        }
    }
}

impl Neg for &Value {
    type Output = Value;

    fn neg(self) -> Value {
        match self {
            &Value::Number(a) => Value::Number(-a),
            _ => panic!("Invalid operation"),
        }
    }
}

impl Value {
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    pub fn is_falsey(&self) -> bool {
        matches!(self, Value::Nil | Value::Boolean(false))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::Str(_))
    }
}

#[derive(Clone, Debug, Default)]
pub struct ValueArray {
    values: Vec<Value>,
}

impl ValueArray {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn write(&mut self, value: Value) -> usize {
        let count = self.values.len();
        self.values.push(value);
        count
    }

    #[cfg(any(feature = "debug_trace_execution", feature = "debug_print_code"))]
    pub fn print_value(&self, which: usize) {
        print!("{}", self.values[which]);
    }

    pub fn read_value(&self, which: usize) -> &Value {
        &self.values[which]
    }
}
