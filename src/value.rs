use std::fmt::{Display, Formatter};
use std::ops::{Add, Div, Mul, Neg, Sub};

use crate::chunk::*;
use crate::function::*;

#[derive(PartialEq, PartialOrd)]
pub enum Value {
    Boolean(bool),
    Number(f64),
    Nil,
    Str(String),
    Func(Function),
}

impl Clone for Value {
    fn clone(&self) -> Self {
        match self {
            Value::Boolean(b) => Value::Boolean(*b),
            Value::Number(n) => Value::Number(*n),
            Value::Nil => Value::Nil,
            Value::Str(s) => Value::Str(s.clone()),
            Value::Func(f) => Value::Func(f.clone()),
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
        }
    }
}

impl Add for Value {
    type Output = Value;

    fn add(self, other: Value) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a + b),
            _ => panic!("Invalid operation"),
        }
    }
}

impl Sub for Value {
    type Output = Value;

    fn sub(self, other: Value) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a - b),
            _ => panic!("Invalid operation"),
        }
    }
}

impl Mul for Value {
    type Output = Value;

    fn mul(self, other: Value) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a * b),
            _ => panic!("Invalid operation"),
        }
    }
}

impl Div for Value {
    type Output = Value;

    fn div(self, other: Value) -> Value {
        match (self, other) {
            (Value::Number(a), Value::Number(b)) => Value::Number(a / b),
            _ => panic!("Invalid operation"),
        }
    }
}

impl Neg for Value {
    type Output = Value;

    fn neg(self) -> Value {
        match self {
            Value::Number(a) => Value::Number(-a),
            _ => panic!("Invalid operation"),
        }
    }
}

impl Value {
    pub fn is_number(&self) -> bool {
        matches!(self, Value::Number(_))
    }

    pub fn is_falsy(&self) -> bool {
        matches!(self, Value::Nil | Value::Boolean(false))
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::Str(_))
    }
}

#[derive(Clone)]
pub struct ValueArray {
    values: Vec<Value>,
}

impl ValueArray {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn write(&mut self, value: Value) -> usize {
        /*
         * String interning?
        if let Value::Str(s) = value.clone() {
            for (i, v) in self.values.iter().enumerate() {
                if let Value::Str(t) = v {
                    if t == &s {
                        return i;
                    }
                }
            }
        }
        */
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
