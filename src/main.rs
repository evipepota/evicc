use std::cell::RefCell;
use std::env;
use std::process;
use std::rc::Rc;

mod codegen;
mod parser;
mod tokenizer;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("args error");
        process::exit(1);
    }

    let tokens = &args[1][..];
    let token = Rc::new(RefCell::new(tokenizer::tokenizer(tokens)));
    let (code, max_offset) = parser::program(&mut token.borrow_mut(), &mut None);

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    println!("  push rbp");
    println!("  mov rbp, rsp");
    println!("  sub rsp, {}", max_offset);

    for node in code {
        codegen::gen(node);
        println!("  pop rax");
    }

    println!("  mov rsp, rbp");
    println!("  pop rbp");
    println!("  ret");
}
