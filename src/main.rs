use std::cell::RefCell;
use std::env;
use std::process;
use std::rc::Rc;

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
    let node = parser::expr(&mut token.borrow_mut());

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    parser::gen(node);

    println!("  pop rax");
    println!("  ret");
}
