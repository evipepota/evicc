use std::cell::RefCell;
use std::env;
use std::process;
use std::rc::Rc;

mod tokenizer;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("args error");
        process::exit(1);
    }

    let tokens = &args[1][..];
    let token = Rc::new(RefCell::new(tokenizer::tokenizer(tokens)));

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    println!(
        "  mov rax, {}",
        tokenizer::expect_number(&mut token.borrow_mut())
    );

    while !tokenizer::at_eof(&token.borrow()) {
        if tokenizer::consume('+', &mut token.borrow_mut()) {
            println!(
                "  add rax, {}",
                tokenizer::expect_number(&mut token.borrow_mut())
            );
            continue;
        }

        tokenizer::expect('-', &mut token.borrow_mut());
        println!(
            "  sub rax, {}",
            tokenizer::expect_number(&mut token.borrow_mut())
        );
    }

    println!("  ret");
}
