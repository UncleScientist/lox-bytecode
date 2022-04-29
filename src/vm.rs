use std::rc::Rc;

use super::*;
use crate::{chunk::*, compiler::*, value::*};

pub struct VM {
    ip: usize,
    stack: Vec<Value>,
    chunk: Rc<Chunk>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            ip: 0,
            stack: Vec::new(),
            chunk: Rc::new(Chunk::new()),
        }
    }

    pub fn interpret(&mut self, source: &str) -> Result<(), InterpretResult> {
        let mut chunk = Chunk::new();
        let mut compiler = Compiler::new(&mut chunk);
        compiler.compile(source)?;

        self.ip = 0;
        self.chunk = Rc::new(chunk);
        self.run()
    }

    fn run(&mut self) -> Result<(), InterpretResult> {
        loop {
            #[cfg(feature = "debug_trace_execution")]
            {
                print!("          ");
                for slot in &self.stack {
                    print!("[ {slot} ]");
                }
                println!();
                self.chunk.disassemble_instruction(self.ip);
            }

            let instruction = self.read_byte();
            match instruction {
                OpCode::Pop => {
                    self.pop();
                }
                OpCode::Print => {
                    println!("{}", self.pop());
                }
                OpCode::Return => {
                    return Ok(());
                }
                OpCode::Constant => {
                    let constant = self.read_constant().clone();
                    self.stack.push(constant);
                }
                OpCode::Nil => self.stack.push(Value::Nil),
                OpCode::True => self.stack.push(Value::Boolean(true)),
                OpCode::False => self.stack.push(Value::Boolean(false)),
                OpCode::Equal => {
                    let b = self.pop();
                    let a = self.pop();
                    self.stack.push(Value::Boolean(a == b));
                }
                OpCode::Greater => self.binary_op(|a, b| Value::Boolean(a > b))?,
                OpCode::Less => self.binary_op(|a, b| Value::Boolean(a < b))?,
                OpCode::Add => self.binary_op(|a, b| a + b)?,
                OpCode::Subtract => self.binary_op(|a, b| a - b)?,
                OpCode::Multiply => self.binary_op(|a, b| a * b)?,
                OpCode::Divide => self.binary_op(|a, b| a / b)?,
                OpCode::Not => {
                    let value = self.pop();
                    self.stack.push(Value::Boolean(value.is_falsy()))
                }
                OpCode::Negate => {
                    if !self.peek(0).is_number() {
                        return self.runtime_error(&"Operand must be a number.");
                    }

                    let value = self.pop();
                    self.stack.push(-value);
                }
            }
        }
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().unwrap()
    }

    fn peek(&self, distance: usize) -> &Value {
        &self.stack[self.stack.len() - distance - 1]
    }

    fn reset_stack(&mut self) {
        self.stack.clear();
    }

    fn read_byte(&mut self) -> OpCode {
        let val: OpCode = self.chunk.read(self.ip).into();
        self.ip += 1;
        val
    }

    fn read_constant(&mut self) -> &Value {
        let index = self.chunk.read(self.ip) as usize;
        self.ip += 1;
        self.chunk.get_constant(index)
    }

    fn binary_op(&mut self, op: fn(a: Value, b: Value) -> Value) -> Result<(), InterpretResult> {
        if self.peek(0).is_string() && self.peek(1).is_string() {
            self.concatenate()
        } else if self.peek(0).is_number() && self.peek(1).is_number() {
            let b = self.pop();
            let a = self.pop();
            self.stack.push(op(a, b));
            Ok(())
        } else {
            self.runtime_error(&"Operands must be two numbers or two strings.")
        }
    }

    fn concatenate(&mut self) -> Result<(), InterpretResult> {
        let b = self.pop();
        let a = self.pop();
        self.stack.push(Value::Str(format!("{a}{b}")));
        Ok(())
    }

    fn runtime_error<T: ToString>(&mut self, err_msg: &T) -> Result<(), InterpretResult> {
        let line = self.chunk.get_line(self.ip - 1);
        eprintln!("{}", err_msg.to_string());
        eprintln!("[line {line}] in script");
        self.reset_stack();

        Err(InterpretResult::RuntimeError)
    }
}
