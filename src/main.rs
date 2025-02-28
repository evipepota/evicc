use std::cell::RefCell;
use std::env;
use std::process;
use std::rc::Rc;

mod codegen;
mod parser;
mod tokenizer;
mod util;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("args error");
        process::exit(1);
    }

    let tokens = &args[1][..];
    let token = Rc::new(RefCell::new(tokenizer::tokenizer(tokens)));
    let codes = parser::program(&mut token.borrow_mut());

    println!(".intel_syntax noprefix");
    println!(".globl main");

    for (args, code, offset, function_name) in codes {
        println!("{}:", function_name);
        println!("  push rbp");
        println!("  mov rbp, rsp");
        println!("  sub rsp, {}", offset);

        let regs = vec!["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
        let mut i = 0;
        for arg in args {
            println!("  mov rax, rbp");
            println!("  sub rax, {}", arg.offset);
            println!("  mov [rax], {}", regs[i]);
            i += 1;
        }

        for node in code {
            codegen::gen(node);
            println!("  pop rax");
        }
    }

    println!("  mov rsp, rbp");
    println!("  pop rbp");
    println!("  ret");
}
