use crate::value::*;

pub enum OpCode {
    Constant = 0,
    Return,
    Negate,
    Add,
    Subtract,
    Multiply,
    Divide,
    Nil,
    True,
    False,
    Not,
    Equal,
    Greater,
    Less,
    Print,
    Pop,
    DefineGlobal,
    GetGlobal,
    SetGlobal,
    GetLocal,
    SetLocal,
    JumpIfFalse,
    Jump,
    Loop,
    Call,
    Closure,
    GetUpvalue,
    SetUpvalue,
    CloseUpvalue,
}

#[derive(Clone, Debug, Default)]
pub struct Chunk {
    code: Vec<u8>,
    lines: Vec<usize>,
    constants: ValueArray,
}

#[cfg(any(feature = "debug_trace_execution", feature = "debug_print_code"))]
#[derive(PartialEq)]
enum JumpStyle {
    Forwards,
    Backwards,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            lines: Vec::new(),
            constants: ValueArray::new(),
        }
    }

    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn write_at(&mut self, offset: usize, byte: u8) {
        self.code[offset] = byte;
    }

    pub fn read(&self, ip: usize) -> u8 {
        self.code[ip]
    }

    pub fn get_line(&self, ip: usize) -> usize {
        self.lines[ip]
    }

    pub fn add_constant(&mut self, value: Value) -> Option<u8> {
        let idx = self.constants.write(value);
        u8::try_from(idx).ok()
    }

    pub fn get_constant(&self, index: usize) -> &Value {
        self.constants.read_value(index)
    }

    pub fn count(&self) -> usize {
        self.lines.len()
    }

    pub fn get_jump_offset(&self, offset: usize) -> usize {
        ((self.code[offset] as usize) << 8) | self.code[offset + 1] as usize
    }

    #[cfg(any(feature = "debug_trace_execution", feature = "debug_print_code"))]
    pub fn disassemble<T: Into<String>>(&self, name: T) {
        println!("== {} ==", name.into());

        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    #[cfg(any(feature = "debug_trace_execution", feature = "debug_print_code"))]
    pub fn disassemble_instruction(&self, offset: usize) -> usize {
        use JumpStyle::*;

        print!("{offset:04} ");

        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:4} ", self.lines[offset]);
        }

        let instruction: OpCode = self.code[offset].into();
        match instruction {
            OpCode::Constant => self.constant_instruction("OP_CONSTANT", offset),
            OpCode::Return => self.simple_instruction("OP_RETURN", offset),
            OpCode::Negate => self.simple_instruction("OP_NEGATE", offset),
            OpCode::Add => self.simple_instruction("OP_ADD", offset),
            OpCode::Subtract => self.simple_instruction("OP_SUBTRACT", offset),
            OpCode::Multiply => self.simple_instruction("OP_MULTIPLY", offset),
            OpCode::Divide => self.simple_instruction("OP_DIVIDE", offset),
            OpCode::Nil => self.simple_instruction("OP_NIL", offset),
            OpCode::True => self.simple_instruction("OP_TRUE", offset),
            OpCode::False => self.simple_instruction("OP_FALSE", offset),
            OpCode::Not => self.simple_instruction("OP_NOT", offset),
            OpCode::Equal => self.simple_instruction("OP_EQUAL", offset),
            OpCode::Greater => self.simple_instruction("OP_GREATER", offset),
            OpCode::Less => self.simple_instruction("OP_LESS", offset),
            OpCode::Print => self.simple_instruction("OP_PRINT", offset),
            OpCode::Pop => self.simple_instruction("OP_POP", offset),
            OpCode::DefineGlobal => self.constant_instruction("OP_DEFINE_GLOBAL", offset),
            OpCode::GetGlobal => self.constant_instruction("OP_GET_GLOBAL", offset),
            OpCode::SetGlobal => self.constant_instruction("OP_SET_GLOBAL", offset),
            OpCode::GetLocal => self.byte_instruction("OP_GET_LOCAL", offset),
            OpCode::SetLocal => self.byte_instruction("OP_SET_LOCAL", offset),
            OpCode::JumpIfFalse => self.jump_instruction("OP_JUMP_IF_FALSE", Forwards, offset),
            OpCode::Jump => self.jump_instruction("OP_JUMP", Forwards, offset),
            OpCode::Loop => self.jump_instruction("OP_LOOP", Backwards, offset),
            OpCode::Call => self.byte_instruction("OP_CALL", offset),
            OpCode::Closure => {
                let mut i = offset + 1;
                let constant = self.code[i];
                i += 1;
                print!("{:-16} {constant:4} ", "OP_CLOSURE");
                self.constants.print_value(constant as usize);
                println!();
                if let Value::Func(function) = self.constants.read_value(constant as usize) {
                    for _ in 0..function.upvalues() {
                        let is_local = if self.code[i] == 0 {
                            "upvalue"
                        } else {
                            "local"
                        };
                        i += 1;
                        let index = self.code[i];
                        i += 1;
                        println!("{:04}      |                     {is_local} {index}", i - 2);
                    }
                } else {
                    panic!("No function at position {constant}");
                }
                i
            }
            OpCode::GetUpvalue => self.byte_instruction("OP_GET_UPVALUE", offset),
            OpCode::SetUpvalue => self.byte_instruction("OP_SET_UPVALUE", offset),
            OpCode::CloseUpvalue => self.simple_instruction("OP_CLOSE_UPVALUE", offset),
        }
    }

    #[cfg(any(feature = "debug_trace_execution", feature = "debug_print_code"))]
    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{name}");
        offset + 1
    }

    #[cfg(any(feature = "debug_trace_execution", feature = "debug_print_code"))]
    fn byte_instruction(&self, name: &str, offset: usize) -> usize {
        let slot = self.code[offset + 1];
        println!("{name:-16} {slot:4}");
        offset + 2
    }

    #[cfg(any(feature = "debug_trace_execution", feature = "debug_print_code"))]
    fn jump_instruction(&self, name: &str, forward_jump: JumpStyle, offset: usize) -> usize {
        let jump = self.get_jump_offset(offset + 1);
        let jump_to = if forward_jump == JumpStyle::Forwards {
            offset + 3 + jump
        } else {
            offset + 3 - jump
        };
        println!("{name:-16} {offset:4} -> {jump_to}");
        offset + 3
    }

    #[cfg(any(feature = "debug_trace_execution", feature = "debug_print_code"))]
    fn constant_instruction(&self, name: &str, offset: usize) -> usize {
        let constant = self.code[offset + 1];
        print!("{name:-16} {constant:4} '");
        self.constants.print_value(constant as usize);
        println!("'");
        offset + 2
    }
}

impl From<u8> for OpCode {
    fn from(code: u8) -> Self {
        match code {
            0 => OpCode::Constant,
            1 => OpCode::Return,
            2 => OpCode::Negate,
            3 => OpCode::Add,
            4 => OpCode::Subtract,
            5 => OpCode::Multiply,
            6 => OpCode::Divide,
            7 => OpCode::Nil,
            8 => OpCode::True,
            9 => OpCode::False,
            10 => OpCode::Not,
            11 => OpCode::Equal,
            12 => OpCode::Greater,
            13 => OpCode::Less,
            14 => OpCode::Print,
            15 => OpCode::Pop,
            16 => OpCode::DefineGlobal,
            17 => OpCode::GetGlobal,
            18 => OpCode::SetGlobal,
            19 => OpCode::GetLocal,
            20 => OpCode::SetLocal,
            21 => OpCode::JumpIfFalse,
            22 => OpCode::Jump,
            23 => OpCode::Loop,
            24 => OpCode::Call,
            25 => OpCode::Closure,
            26 => OpCode::GetUpvalue,
            27 => OpCode::SetUpvalue,
            28 => OpCode::CloseUpvalue,
            _ => unimplemented!("Invalid opcode"),
        }
    }
}

impl From<OpCode> for u8 {
    fn from(code: OpCode) -> Self {
        code as u8
    }
}
