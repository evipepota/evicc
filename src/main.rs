use std::env;
use std::process;

fn parse_number(input: &mut &str) -> i64 {
    let mut chars = input.chars();
    let mut num_str = String::new();

    while let Some(c) = chars.next() {
        if c.is_digit(10) {
            num_str.push(c);
        } else {
            break;
        }
    }

    if num_str.is_empty() {
        eprintln!("integer error");
        process::exit(1);
    }

    *input = &input[num_str.len()..];
    num_str.parse().expect("error")
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("args error");
        process::exit(1);
    }

    let tokens = &args[1][..];

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    let mut rest = tokens;
    let mut num = parse_number(&mut rest);
    println!("  mov rax, {}", num);

    while !rest.is_empty() {
        match rest.chars().next().unwrap() {
            '+' => {
                rest = &rest[1..];
                num = parse_number(&mut rest);
                println!("  add rax, {}", num);
            }
            '-' => {
                rest = &rest[1..];
                num = parse_number(&mut rest);
                println!("  sub rax, {}", num);
            }
            c => {
                eprintln!("string error: {}", c);
                process::exit(1);
            }
        }
    }

    println!("  ret");
}
