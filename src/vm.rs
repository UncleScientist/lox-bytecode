use std::cell::RefCell;
use std::collections::{hash_map::Entry, HashMap};
use std::rc::Rc;

use crate::{chunk::*, closure::*, compiler::*, error::*, native::*, value::*};

pub struct VM {
    stack: Vec<Value>,
    frames: Vec<CallFrame>,
    globals: HashMap<String, Value>,
}

#[derive(Debug)]
struct CallFrame {
    function: usize, // index into VM.stack
    ip: RefCell<usize>,
    slots: usize,
}

impl CallFrame {
    fn inc(&self, amount: usize) {
        *self.ip.borrow_mut() += amount;
    }

    fn dec(&self, amount: usize) {
        *self.ip.borrow_mut() -= amount;
    }
}

impl VM {
    pub fn new() -> Self {
        let mut vm = Self {
            stack: Vec::new(),
            frames: Vec::new(),
            globals: HashMap::new(),
        };
        let f: Rc<dyn NativeFunc> = Rc::new(NativeClock {});
        vm.define_native("clock", &f);
        vm
    }

    pub fn interpret(&mut self, source: &str) -> Result<(), InterpretResult> {
        let mut compiler = Compiler::new();
        let function = compiler.compile(source)?;

        self.stack
            .push(Value::Closure(Rc::new(Closure::new(Rc::new(function)))));
        self.call(0);
        let result = self.run();
        self.stack.pop();

        result
    }

    fn current_frame(&self) -> &CallFrame {
        self.frames.last().unwrap()
    }

    fn ip(&self) -> usize {
        *self.current_frame().ip.borrow()
    }

    fn chunk(&self) -> Rc<Chunk> {
        let position = self.current_frame().function;
        if let Value::Closure(c) = &self.stack[position] {
            c.get_chunk()
        } else {
            panic!("no chunk");
        }
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
                self.chunk().disassemble_instruction(self.ip());
            }

            let instruction: OpCode = self.read_byte().into();
            match instruction {
                OpCode::GetUpvalue | OpCode::SetUpvalue => todo!(),
                OpCode::Closure => {
                    let constant = self.read_constant().clone();
                    if let Value::Func(function) = constant {
                        let closure = Closure::new(function);
                        self.stack.push(Value::Closure(Rc::new(closure)));
                    } else {
                        panic!("Tried to read function from constant table but got {constant:?}");
                    }
                }
                OpCode::Call => {
                    let arg_count = self.read_byte() as usize;
                    if !self.call_value(arg_count) {
                        return Err(InterpretResult::RuntimeError);
                    }
                }
                OpCode::Loop => {
                    let offset = self.read_short();
                    self.current_frame().dec(offset);
                }
                OpCode::Jump => {
                    let offset = self.read_short();
                    self.current_frame().inc(offset);
                }
                OpCode::JumpIfFalse => {
                    let offset = self.read_short();
                    if self.peek(0).is_falsey() {
                        self.current_frame().inc(offset);
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
                    let slot_offset = self.current_frame().slots;
                    self.stack.push(self.stack[slot_offset + slot].clone());
                }
                OpCode::SetLocal => {
                    let slot = self.read_byte() as usize;
                    let slot_offset = self.current_frame().slots;
                    self.stack[slot_offset + slot] = self.peek(0).clone();
                }
                OpCode::Print => {
                    println!("{}", self.pop());
                }
                OpCode::Return => {
                    let result = self.pop();
                    let prev_frame = self.frames.pop().unwrap();
                    if self.frames.is_empty() {
                        self.pop();
                        return Ok(());
                    }
                    self.stack.truncate(prev_frame.slots);
                    self.stack.push(result);
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
                    self.stack.push(Value::Boolean(value.is_falsey()))
                }
                OpCode::Negate => {
                    if !self.peek(0).is_number() {
                        return self.runtime_error("Operand must be a number.");
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

    fn call(&mut self, arg_count: usize) -> bool {
        let arity = if let Value::Closure(callee) = self.peek(arg_count) {
            callee.arity()
        } else {
            panic!("tried to call a non-function: {:?}", self.peek(arg_count));
        };
        if arity != arg_count {
            let _ = self.runtime_error(&format!("Expected {arity} arguments but got {arg_count}."));
            return false;
        }

        if self.frames.len() == 256 {
            let _ = self.runtime_error("Stack overflow.");
            return false;
        }

        self.frames.push(CallFrame {
            function: self.stack.len() - arg_count - 1,
            ip: RefCell::new(0),
            slots: self.stack.len() - arg_count - 1,
        });

        true
    }

    fn call_value(&mut self, arg_count: usize) -> bool {
        let callee = self.peek(arg_count);
        let success = match callee {
            Value::Closure(_) => {
                return self.call(arg_count);
            }
            Value::Native(f) => {
                let stack_top = self.stack.len();
                let result = f.call(arg_count, &self.stack[stack_top - arg_count..stack_top]);
                self.stack.truncate(stack_top - arg_count + 1);
                self.stack.push(result);
                true
            }
            _ => false,
        };

        if !success {
            let _ = self.runtime_error("Can only call functions and classes.");
        }

        success
    }

    fn reset_stack(&mut self) {
        self.stack.clear();
    }

    fn read_byte(&mut self) -> u8 {
        let val: u8 = self.chunk().read(self.ip());
        self.current_frame().inc(1);
        val
    }

    fn read_short(&mut self) -> usize {
        self.current_frame().inc(2);
        self.chunk().get_jump_offset(self.ip() - 2)
    }

    fn read_constant(&mut self) -> Value {
        let index = self.chunk().read(self.ip()) as usize;
        self.current_frame().inc(1);
        self.chunk().get_constant(index).clone()
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
            println!("{:?} and {:?}", self.peek(0), self.peek(1));
            self.runtime_error("Operands must be two numbers or two strings.")
        }
    }

    fn concatenate(&mut self) -> Result<(), InterpretResult> {
        let b = self.pop();
        let a = self.pop();
        self.stack.push(Value::Str(format!("{a}{b}")));
        Ok(())
    }

    fn runtime_error<T: Into<String>>(&mut self, err_msg: T) -> Result<(), InterpretResult> {
        eprintln!("{}", err_msg.into());
        for frame in self.frames.iter().rev() {
            if let Value::Closure(closure) = &self.stack[frame.function] {
                let instruction = *frame.ip.borrow() - 1;
                let line = closure.get_chunk().get_line(instruction);
                eprintln!("[line {line}] in {}", closure.stack_name());
            } else {
                panic!("tried to get a stack trace of a non-function");
            }
        }
        self.reset_stack();

        Err(InterpretResult::RuntimeError)
    }

    fn define_native<T: Into<String>>(&mut self, name: T, function: &Rc<dyn NativeFunc>) {
        self.globals
            .insert(name.into(), Value::Native(Rc::clone(function)));
    }
}
