use std::env::args;
use std::io::{self, stdout, BufRead, Write};

mod compiler;
mod scanner;
mod value;

mod vm;
use vm::*;

mod chunk;
use chunk::*;

fn main() {
    let args: Vec<String> = args().collect();
    let mut vm = VM::new();

    match args.len() {
        1 => repl(&mut vm),
        2 => run_file(&mut vm, &args[1]).expect("Could not run file"),
        _ => {
            println!("Usage: lox-bytecode [script]");
            std::process::exit(64);
        }
    }

    vm.free();
}

fn repl(vm: &mut VM) {
    let stdin = io::stdin();
    print!("> ");
    let _ = stdout().flush();
    for line in stdin.lock().lines() {
        if let Ok(line) = line {
            if line.is_empty() {
                break;
            }
            let _ = vm.interpret(&line);
        } else {
            break;
        }
        print!("> ");
        let _ = stdout().flush();
    }
}

fn run_file(vm: &mut VM, path: &str) -> io::Result<()> {
    let buf = std::fs::read_to_string(path)?;
    match vm.interpret(&buf) {
        InterpretResult::CompileError => std::process::exit(65),
        InterpretResult::RuntimeError => std::process::exit(70),
        InterpretResult::Ok => std::process::exit(0),
    }
}
