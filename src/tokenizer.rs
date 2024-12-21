use std::process;
use std::sync::RwLock;

use lazy_static::lazy_static;

lazy_static! {
    static ref USER_INPUT: RwLock<Option<String>> = RwLock::new(None);
}

pub enum TokenKind {
    TkReserved,
    TkNum,
    TkEof,
}

pub struct Token {
    kind: TokenKind,
    next: Option<Box<Token>>,
    val: Option<i32>,
    str: String,
    loc: usize, // token location in input
}

impl Token {
    fn new(kind: TokenKind, val: Option<i32>, str: String, loc: usize) -> Self {
        Token {
            kind,
            next: None,
            val,
            str,
            loc,
        }
    }
}

fn error(msg: &str) -> ! {
    eprintln!("Error: {}", msg);
    process::exit(1);
}

fn error_at(loc: usize, msg: &str) -> ! {
    if let Some(ref user_input) = *USER_INPUT.write().unwrap() {
        let input = &user_input;
        eprintln!("{}", input);
        eprintln!("{:>width$}^ {}", "", msg, width = loc);
    } else {
        eprintln!("Error: {}", msg);
    }
    process::exit(1);
}

pub fn consume(op: char, token: &mut Option<Box<Token>>) -> bool {
    if let Some(current) = token {
        if let TokenKind::TkReserved = current.kind {
            if current.str.chars().next() == Some(op) {
                *token = current.next.take();
                return true;
            }
        }
    }
    false
}

pub fn expect(op: char, token: &mut Option<Box<Token>>) {
    if !consume(op, token) {
        if let Some(current) = token {
            error_at(current.loc, &format!("expected token is '{}'", op));
        }
    }
}

pub fn expect_number(token: &mut Option<Box<Token>>) -> i32 {
    if let Some(current) = token {
        if let TokenKind::TkNum = current.kind {
            let val = current.val.unwrap();
            *token = current.next.take();
            return val;
        }
        error_at(current.loc, "expected number");
    } else {
        error("unexpected error");
    }
}

pub fn at_eof(token: &Option<Box<Token>>) -> bool {
    if let Some(token) = token {
        matches!(token.kind, TokenKind::TkEof)
    } else {
        false
    }
}

fn new_token(kind: TokenKind, cur: &mut Token, str: String, loc: usize) -> &mut Token {
    let tok = Token::new(kind, None, str.to_string(), loc);
    cur.next = Some(Box::new(tok));
    cur.next.as_mut().unwrap()
}

pub fn tokenizer(input: &str) -> Option<Box<Token>> {
    let mut head = Token::new(TokenKind::TkEof, None, String::new(), 0);
    let mut cur = &mut head;
    let mut chars = input.chars().peekable();

    *USER_INPUT.write().unwrap() = Some(input.to_string());

    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }

        if c == '+' || c == '-' {
            cur = new_token(
                TokenKind::TkReserved,
                cur,
                c.to_string(),
                input.len() - chars.clone().count(),
            );
            chars.next();
            continue;
        }

        if c.is_digit(10) {
            let mut num_str = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_digit(10) {
                    num_str.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            cur = new_token(
                TokenKind::TkNum,
                cur,
                num_str.to_string(),
                input.len() - chars.clone().count(),
            );
            cur.val = Some(num_str.parse().unwrap());
            continue;
        }

        if cur.loc == 0 {
            error_at(cur.loc, "invalid token");
        } else {
            error_at(cur.loc + 1, "invalid token");
        }
    }

    new_token(TokenKind::TkEof, cur, String::new(), input.len());
    head.next
}
