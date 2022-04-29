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
}

pub struct Chunk {
    code: Vec<u8>,
    lines: Vec<usize>,
    constants: ValueArray,
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

    pub fn disassemble<T: ToString>(&self, name: T) {
        println!("== {} ==", name.to_string());

        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    pub fn disassemble_instruction(&self, offset: usize) -> usize {
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
        }
    }

    fn simple_instruction(&self, name: &str, offset: usize) -> usize {
        println!("{name}");
        offset + 1
    }

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
            _ => unimplemented!("Invalid opcode"),
        }
    }
}

impl From<OpCode> for u8 {
    fn from(code: OpCode) -> Self {
        code as u8
    }
}
