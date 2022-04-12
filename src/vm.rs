use crate::chunk::*;
use crate::value::*;

pub enum InterpretResult {
    Ok,
    // CompileError,
    // RuntimeError,
}

pub struct VM {
    // chunk: Option<Chunk>,
    ip: usize,
}

impl VM {
    pub fn new() -> Self {
        Self { ip: 0 }
    }

    pub fn free(&mut self) {}

    pub fn interpret(&mut self, chunk: &Chunk) -> InterpretResult {
        self.ip = 0;
        self.run(chunk)
    }

    fn run(&mut self, chunk: &Chunk) -> InterpretResult {
        loop {
            #[cfg(feature = "debug_trace_execution")]
            chunk.disassemble_instruction(self.ip);

            let instruction = self.read_byte(chunk);
            match instruction {
                OpCode::OpReturn => {
                    return InterpretResult::Ok;
                }
                OpCode::OpConstant => {
                    let constant = self.read_constant(chunk);
                    println!("{constant}");
                }
            }
        }
    }

    fn read_byte(&mut self, chunk: &Chunk) -> OpCode {
        let val: OpCode = chunk.read(self.ip).into();
        self.ip += 1;
        val
    }

    fn read_constant(&mut self, chunk: &Chunk) -> Value {
        let index = chunk.read(self.ip) as usize;
        self.ip += 1;
        chunk.get_constant(index)
    }
}
