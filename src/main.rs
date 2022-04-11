mod chunk;
use chunk::*;

fn main() {
    let mut chunk = Chunk::new();

    chunk.write_opcode(OpCode::OpReturn);
    chunk.disassemble("test chunk");

    chunk.free();
}
