use std::cell::RefCell;
use std::collections::{hash_map::Entry, HashMap};
use std::rc::Rc;

use crate::{
    bound_method::*, chunk::*, class::*, closure::*, compiler::*, error::*, instance::*, native::*,
    value::*,
};

pub struct VM {
    stack: Vec<Rc<RefCell<Value>>>,
    frames: Vec<CallFrame>,
    globals: HashMap<String, Value>,
}

#[derive(Debug)]
struct CallFrame {
    closure: Rc<Closure>, // index into VM.stack
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

        let closure = Rc::new(Closure::new(Rc::new(function)));
        self.stack
            .push(Rc::new(RefCell::new(Value::Closure(Rc::clone(&closure)))));
        self.call(closure, 0);
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

    fn get_upvalue(&self, offset: usize) -> Rc<RefCell<Value>> {
        self.current_frame().closure.get_upvalue(offset)
    }

    fn set_upvalue(&self, offset: usize, value: &Rc<RefCell<Value>>) {
        self.current_frame().closure.modify(offset, value);
    }

    fn capture_upvalue(&self, offset: usize) -> Rc<RefCell<Value>> {
        Rc::clone(&self.stack[offset])
    }

    fn chunk(&self) -> Rc<Chunk> {
        self.current_frame().closure.get_chunk()
    }

    fn run(&mut self) -> Result<(), InterpretResult> {
        loop {
            #[cfg(feature = "debug_trace_execution")]
            {
                print!("          ");
                for slot in &self.stack {
                    print!("[ {} ]", slot.borrow());
                }
                println!();
                self.chunk().disassemble_instruction(self.ip());
            }

            let instruction: OpCode = self.read_byte().into();
            match instruction {
                OpCode::SuperInvoke => {
                    let constant = self.read_constant().clone();
                    let method_name = if let Value::Str(s) = constant {
                        s
                    } else {
                        panic!("Unable to get field name from table");
                    };

                    let arg_count = self.read_byte() as usize;
                    let popped_value = self.pop().borrow().clone();
                    let superclass = if let Value::Class(klass) = popped_value {
                        klass
                    } else {
                        return Err(InterpretResult::RuntimeError);
                    };

                    if !self.invoke_from_class(superclass, &method_name, arg_count) {
                        return Err(InterpretResult::RuntimeError);
                    }
                }
                OpCode::GetSuper => {
                    let constant = self.read_constant().clone();
                    let name = if let Value::Str(s) = constant {
                        s
                    } else {
                        panic!("Unable to get field name from table");
                    };
                    let popped_value = self.pop().borrow().clone();
                    let superclass = if let Value::Class(klass) = popped_value {
                        klass
                    } else {
                        panic!("no superclass");
                    };
                    if !self.bind_method(superclass, &name) {
                        return Err(InterpretResult::RuntimeError);
                    }
                }
                OpCode::Inherit => {
                    let superclass_value = self.peek(1).borrow().clone();
                    let superclass = if let Value::Class(c) = superclass_value {
                        c
                    } else {
                        return self.runtime_error("Superclass must be a class.");
                    };
                    let subclass = if let Value::Class(c) = self.peek(0).borrow().clone() {
                        c
                    } else {
                        panic!("No subclass found on stack.");
                    };

                    subclass.copy_methods(&superclass);

                    self.pop();
                }
                OpCode::Invoke => {
                    let constant = self.read_constant().clone();
                    let method_name = if let Value::Str(s) = constant {
                        s
                    } else {
                        panic!("Unable to get field name from table");
                    };

                    let arg_count = self.read_byte() as usize;
                    if !self.invoke(method_name.as_str(), arg_count) {
                        return Err(InterpretResult::RuntimeError);
                    }
                }
                OpCode::Method => {
                    let constant = self.read_constant().clone();
                    let method_name = if let Value::Str(s) = constant {
                        s
                    } else {
                        panic!("Unable to get class name from table");
                    };
                    self.define_method(&method_name);
                }
                OpCode::SetProperty => {
                    let instance = if let Value::Instance(i) = self.peek(1).borrow().clone() {
                        Some(i)
                    } else {
                        None
                    };

                    if instance.is_none() {
                        return self.runtime_error("Only instances have fields.");
                    }

                    let constant = self.read_constant().clone();
                    let field_name = if let Value::Str(s) = constant {
                        s
                    } else {
                        panic!("Unable to get class name from table");
                    };

                    let value = self.pop();
                    instance
                        .unwrap()
                        .set_field(field_name, &value.borrow().clone());

                    self.pop(); // Instance
                    self.push(value.borrow().clone());
                }
                OpCode::GetProperty => {
                    let instance = if let Value::Instance(i) = self.peek(0).borrow().clone() {
                        Some(i)
                    } else {
                        None
                    };
                    // xyzzy

                    if instance.is_none() {
                        return self.runtime_error("Only instances have properties.");
                    }

                    let constant = self.read_constant().clone();
                    let field_name = if let Value::Str(s) = constant {
                        s
                    } else {
                        panic!("Unable to get field name from table");
                    };

                    if let Some(value) = instance.as_ref().unwrap().get_field(&field_name) {
                        self.pop(); // Instance
                        self.push(value.clone());
                    } else if !self.bind_method(instance.unwrap().get_class(), &field_name) {
                        return self.runtime_error(&format!("Undefined property '{field_name}'."));
                    }
                }
                OpCode::Class => {
                    let constant = self.read_constant().clone();
                    let class_string = if let Value::Str(s) = constant {
                        s
                    } else {
                        panic!("Unable to get class name from table");
                    };
                    self.push(Value::Class(Rc::new(Class::new(class_string))));
                }
                OpCode::GetUpvalue => {
                    let slot = self.read_byte() as usize;
                    self.stack.push(Rc::clone(&self.get_upvalue(slot)));
                }
                OpCode::SetUpvalue => {
                    let slot = self.read_byte() as usize;
                    let value = self.peek(0);
                    self.set_upvalue(slot, value);
                }
                OpCode::Closure => {
                    let constant = self.read_constant().clone();
                    if let Value::Func(function) = constant {
                        let upvalue_count = function.upvalues();
                        let closure = Closure::new(function);
                        for _ in 0..upvalue_count {
                            let is_local = self.read_byte() != 0;
                            let index = self.read_byte() as usize;
                            let captured = if is_local {
                                let offset = self.current_frame().slots + index;
                                self.capture_upvalue(offset)
                            } else {
                                self.get_upvalue(index)
                            };
                            closure.push_upvalue(&captured);
                        }
                        self.push(Value::Closure(Rc::new(closure)));
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
                    if self.peek(0).borrow().is_falsey() {
                        self.current_frame().inc(offset);
                    }
                }
                OpCode::DefineGlobal => {
                    let constant = self.read_constant().clone();
                    if let Value::Str(s) = constant {
                        let p = self.pop();
                        self.globals.insert(s, p.borrow().clone());
                    } else {
                        panic!("Unable to read constant from table");
                    }
                }
                OpCode::GetGlobal => {
                    let constant = self.read_constant().clone();
                    if let Value::Str(s) = constant {
                        if let Some(v) = self.globals.get(&s) {
                            let u = v.clone();
                            self.push(u);
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
                        let p = self.peek(0).borrow().clone();
                        if let Entry::Occupied(mut o) = self.globals.entry(s.clone()) {
                            *o.get_mut() = p;
                        } else {
                            return self.runtime_error(&format!("Undefined variable '{s}'."));
                        }
                    }
                }
                OpCode::CloseUpvalue | OpCode::Pop => {
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
                    println!("{}", self.pop().borrow());
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
                    self.push(constant);
                }
                OpCode::Nil => self.push(Value::Nil),
                OpCode::True => self.push(Value::Boolean(true)),
                OpCode::False => self.push(Value::Boolean(false)),
                OpCode::Equal => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(Value::Boolean(a == b));
                }
                OpCode::Greater => self.binary_op(|a, b| Value::Boolean(a > b))?,
                OpCode::Less => self.binary_op(|a, b| Value::Boolean(a < b))?,
                OpCode::Add => self.binary_op(|a, b| a + b)?,
                OpCode::Subtract => self.binary_op(|a, b| a - b)?,
                OpCode::Multiply => self.binary_op(|a, b| a * b)?,
                OpCode::Divide => self.binary_op(|a, b| a / b)?,
                OpCode::Not => {
                    let value = self.pop().borrow().clone();
                    self.push(Value::Boolean(value.is_falsey()))
                }
                OpCode::Negate => {
                    if !self.peek(0).borrow().is_number() {
                        return self.runtime_error("Operand must be a number.");
                    }

                    let value = self.pop().borrow().clone();
                    self.push(-&value);
                }
            }
        }
    }

    fn define_method(&mut self, name: &str) {
        let method = self.peek(0).borrow().clone();
        let klass = if let Value::Class(klass) = self.peek(1).borrow().clone() {
            Some(klass)
        } else {
            panic!("compiler bug - no class found at stack[-2]");
        };

        if name == "init" {
            if let Value::Closure(closure) = method {
                klass.unwrap().set_init_method(closure)
            } else {
                panic!("method should have been a closure");
            }
        } else {
            klass.unwrap().add_method(name, &method);
        }
        self.pop();
    }

    fn push(&mut self, value: Value) {
        self.stack.push(Rc::new(RefCell::new(value)));
    }

    fn pop(&mut self) -> Rc<RefCell<Value>> {
        self.stack.pop().unwrap()
    }

    fn peek(&self, distance: usize) -> &Rc<RefCell<Value>> {
        &self.stack[self.stack.len() - distance - 1]
    }

    fn call(&mut self, closure: Rc<Closure>, arg_count: usize) -> bool {
        let arity = closure.arity();
        if arity != arg_count {
            let _ = self.runtime_error(&format!("Expected {arity} arguments but got {arg_count}."));
            return false;
        }

        if self.frames.len() == 256 {
            let _ = self.runtime_error("Stack overflow.");
            return false;
        }

        self.frames.push(CallFrame {
            closure: Rc::clone(&closure),
            ip: RefCell::new(0),
            slots: self.stack.len() - arg_count - 1,
        });

        true
    }

    fn call_value(&mut self, arg_count: usize) -> bool {
        let callee = self.peek(arg_count).borrow().clone();
        let success = match callee {
            Value::Bound(method) => {
                let stack_top = self.stack.len();
                self.stack[stack_top - arg_count - 1] =
                    Rc::new(RefCell::new(method.get_receiver()));
                return self.call(method.get_closure(), arg_count);
            }

            Value::Class(klass) => {
                let stack_top = self.stack.len();
                let init = klass.get_init_method();
                self.stack[stack_top - arg_count - 1] =
                    Rc::new(RefCell::new(Value::Instance(Rc::new(Instance::new(klass)))));
                if let Some(initializer) = init {
                    self.call(initializer, arg_count)
                } else if arg_count != 0 {
                    let _ =
                        self.runtime_error(format!("Expected 0 arguments but got {arg_count}."));
                    false
                } else {
                    true
                }
            }

            Value::Closure(closure) => {
                return self.call(closure, arg_count);
            }

            Value::Native(f) => {
                let stack_top = self.stack.len();
                let result = f.call(arg_count, &self.stack[stack_top - arg_count..stack_top]);
                self.stack.truncate(stack_top - (arg_count + 1));
                self.push(result);
                true
            }
            _ => false,
        };

        if !success {
            let _ = self.runtime_error("Can only call functions and classes.");
        }

        success
    }

    fn invoke_from_class(&mut self, klass: Rc<Class>, name: &str, arg_count: usize) -> bool {
        if let Some(closure) = klass.get_method(name) {
            self.call(closure, arg_count)
        } else {
            let _ = self.runtime_error(&format!("Undefined property '{name}'."));
            false
        }
    }

    fn invoke(&mut self, name: &str, arg_count: usize) -> bool {
        let receiver = self.peek(arg_count).borrow().clone();
        if let Value::Instance(instance) = receiver {
            if let Some(value) = instance.get_field(name) {
                let stack_top = self.stack.len();
                self.stack[stack_top - arg_count - 1] = Rc::new(RefCell::new(value));
                self.call_value(arg_count)
            } else {
                self.invoke_from_class(instance.get_class(), name, arg_count)
            }
        } else {
            let _ = self.runtime_error("Only instances have methods.");
            false
        }
    }

    fn bind_method(&mut self, klass: Rc<Class>, name: &str) -> bool {
        if let Some(method) = klass.get_method(name) {
            let value = self.peek(0).borrow().clone();
            let bound = Rc::new(BoundMethod::new(&value, &method));
            self.pop();
            self.push(Value::Bound(bound));
            true
        } else {
            let _ = self.runtime_error(format!("Undefined property '{name}'."));
            false
        }
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

    fn binary_op(&mut self, op: fn(a: &Value, b: &Value) -> Value) -> Result<(), InterpretResult> {
        if self.peek(0).borrow().is_string() && self.peek(1).borrow().is_string() {
            self.concatenate()
        } else if self.peek(0).borrow().is_number() && self.peek(1).borrow().is_number() {
            let b = self.pop();
            let a = self.pop();
            self.push(op(&a.borrow(), &b.borrow()));
            Ok(())
        } else {
            println!("{:?} and {:?}", self.peek(0), self.peek(1));
            self.runtime_error("Operands must be two numbers or two strings.")
        }
    }

    fn concatenate(&mut self) -> Result<(), InterpretResult> {
        let b = self.pop();
        let a = self.pop();
        self.push(Value::Str(format!("{}{}", a.borrow(), b.borrow())));
        Ok(())
    }

    fn runtime_error<T: Into<String>>(&mut self, err_msg: T) -> Result<(), InterpretResult> {
        eprintln!("{}", err_msg.into());
        for frame in self.frames.iter().rev() {
            let instruction = *frame.ip.borrow() - 1;
            let line = frame.closure.get_chunk().get_line(instruction);
            eprintln!("[line {line}] in {}", frame.closure.stack_name());
        }
        self.reset_stack();

        Err(InterpretResult::RuntimeError)
    }

    fn define_native<T: Into<String>>(&mut self, name: T, function: &Rc<dyn NativeFunc>) {
        self.globals
            .insert(name.into(), Value::Native(Rc::clone(function)));
    }
}
