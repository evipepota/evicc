use std::process;

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
}

impl Token {
    fn new(kind: TokenKind, val: Option<i32>, str: String) -> Self {
        Token {
            kind,
            next: None,
            val,
            str,
        }
    }
}

fn error(msg: &str) -> ! {
    eprintln!("{}", msg);
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
        error(&format!("expected token is '{}'", op));
    }
}

pub fn expect_number(token: &mut Option<Box<Token>>) -> i32 {
    if let Some(current) = token {
        if let TokenKind::TkNum = current.kind {
            let val = current.val.unwrap();
            *token = current.next.take();
            return val;
        }
    }
    error("expected number");
}

pub fn at_eof(token: &Option<Box<Token>>) -> bool {
    if let Some(token) = token {
        matches!(token.kind, TokenKind::TkEof)
    } else {
        false
    }
}

fn new_token(kind: TokenKind, cur: &mut Token, str: String) -> &mut Token {
    let tok = Token::new(kind, None, str);
    cur.next = Some(Box::new(tok));
    cur.next.as_mut().unwrap()
}

pub fn tokenizer(input: &str) -> Option<Box<Token>> {
    let mut head = Token::new(TokenKind::TkEof, None, String::new());
    let mut cur = &mut head;
    let mut chars = input.chars().peekable();

    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
            continue;
        }

        if c == '+' || c == '-' {
            cur = new_token(TokenKind::TkReserved, cur, c.to_string());
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
            cur = new_token(TokenKind::TkNum, cur, num_str.clone());
            cur.val = Some(num_str.parse().unwrap());
            continue;
        }

        error("invalid token");
    }

    new_token(TokenKind::TkEof, cur, String::new());
    head.next
}
