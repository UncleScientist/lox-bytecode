mod chunk;
mod value;
use chunk::*;

fn main() {
    let mut chunk = Chunk::new();

    let constant = chunk.add_constant(1.2);
    chunk.write_opcode(OpCode::OpConstant);
    chunk.write(constant);

    chunk.write_opcode(OpCode::OpReturn);
    chunk.disassemble("test chunk");

    chunk.free();
}
