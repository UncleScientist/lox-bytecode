use std::collections::{hash_map::Entry, HashMap};
use std::rc::Rc;

use crate::{chunk::*, compiler::*, error::*, value::*};

pub struct VM {
    ip: usize,
    stack: Vec<Value>,
    chunk: Rc<Chunk>,
    globals: HashMap<String, Value>,
}

impl VM {
    pub fn new() -> Self {
        Self {
            ip: 0,
            stack: Vec::new(),
            chunk: Rc::new(Chunk::new()),
            globals: HashMap::new(),
        }
    }

    pub fn interpret(&mut self, source: &str) -> Result<(), InterpretResult> {
        let mut compiler = Compiler::new();
        let func = compiler.compile(source)?;

        self.ip = 0;
        self.chunk = func.get_chunk();
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

            let instruction: OpCode = self.read_byte().into();
            match instruction {
                OpCode::Loop => {
                    let offset = self.read_short();
                    self.ip -= offset;
                }
                OpCode::Jump => {
                    let offset = self.read_short();
                    self.ip += offset;
                }
                OpCode::JumpIfFalse => {
                    let offset = self.read_short();
                    if self.peek(0).is_falsy() {
                        self.ip += offset;
                    }
                }
                OpCode::DefineGlobal => {
                    let constant = self.read_constant().clone();
                    if let Value::Str(s) = constant {
                        let p = self.pop();
                        self.globals.insert(s, p.clone());
                    } else {
                        panic!("Unable to read constant from table");
                    }
                }
                OpCode::GetGlobal => {
                    let constant = self.read_constant().clone();
                    if let Value::Str(s) = constant {
                        if let Some(v) = self.globals.get(&s) {
                            self.stack.push(v.clone())
                        } else {
                            return self.runtime_error(&format!("Undefined variable {s}."));
                        }
                    } else {
                        panic!("Unable to read constant from table");
                    }
                }
                OpCode::SetGlobal => {
                    let constant = self.read_constant().clone();
                    if let Value::Str(s) = constant {
                        let p = self.peek(0).clone();
                        if let Entry::Occupied(mut o) = self.globals.entry(s.clone()) {
                            *o.get_mut() = p;
                        } else {
                            return self.runtime_error(&format!("Undefined variable '{s}'."));
                        }
                    }
                }
                OpCode::Pop => {
                    self.pop();
                }
                OpCode::GetLocal => {
                    let slot = self.read_byte() as usize;
                    self.stack.push(self.stack[slot].clone());
                }
                OpCode::SetLocal => {
                    let slot = self.read_byte() as usize;
                    self.stack[slot] = self.peek(0).clone();
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

    fn read_byte(&mut self) -> u8 {
        let val: u8 = self.chunk.read(self.ip);
        self.ip += 1;
        val
    }

    fn read_short(&mut self) -> usize {
        self.ip += 2;
        self.chunk.get_jump_offset(self.ip - 2)
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
