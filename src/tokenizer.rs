use crate::util::{error_at, USER_INPUT};

#[derive(PartialEq, Debug)]
pub enum TokenKind {
    TkReserved,
    TkIdent,
    TkNum,
    TkReturn,
    TkIf,
    TkElse,
    TkWhile,
    TkInt,
    TkFor,
    TkSizeof,
    TkEof,
}

pub struct Token {
    pub kind: TokenKind,
    pub next: Option<Box<Token>>,
    pub val: Option<i32>,
    pub str: String,
    pub loc: usize, // token location in input
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

        if c == '=' {
            // consume "==" or "="
            chars.next();
            if let Some(&c) = chars.peek() {
                if c == '=' {
                    cur = new_token(
                        TokenKind::TkReserved,
                        cur,
                        "==".to_string(),
                        input.len() - chars.clone().count(),
                    );
                    chars.next();
                    continue;
                } else {
                    cur = new_token(
                        TokenKind::TkReserved,
                        cur,
                        "=".to_string(),
                        input.len() - chars.clone().count(),
                    );
                    continue;
                }
            }
        }
        if c == '!' {
            // consume "!="
            chars.next();
            if let Some(&c) = chars.peek() {
                if c == '=' {
                    cur = new_token(
                        TokenKind::TkReserved,
                        cur,
                        "!=".to_string(),
                        input.len() - chars.clone().count(),
                    );
                    chars.next();
                    continue;
                } else {
                    error_at(input.len() - chars.clone().count(), "invalid token");
                }
            }
        }
        if c == '<' {
            // consume "<=" or "<"
            chars.next();
            if let Some(&c) = chars.peek() {
                if c == '=' {
                    cur = new_token(
                        TokenKind::TkReserved,
                        cur,
                        "<=".to_string(),
                        input.len() - chars.clone().count(),
                    );
                    chars.next();
                    continue;
                } else {
                    cur = new_token(
                        TokenKind::TkReserved,
                        cur,
                        "<".to_string(),
                        input.len() - chars.clone().count(),
                    );
                    continue;
                }
            }
        }
        if c == '>' {
            // consume ">=" or ">"
            chars.next();
            if let Some(&c) = chars.peek() {
                if c == '=' {
                    cur = new_token(
                        TokenKind::TkReserved,
                        cur,
                        ">=".to_string(),
                        input.len() - chars.clone().count(),
                    );
                    chars.next();
                    continue;
                } else {
                    cur = new_token(
                        TokenKind::TkReserved,
                        cur,
                        ">".to_string(),
                        input.len() - chars.clone().count(),
                    );
                    continue;
                }
            }
        }

        if c == '+'
            || c == '-'
            || c == '*'
            || c == '/'
            || c == '('
            || c == ')'
            || c == '{'
            || c == '}'
            || c == ';'
            || c == ','
            || c == '&'
        {
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

        if c.is_alphabetic() {
            let mut ident_str = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_alphanumeric() {
                    ident_str.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            if ident_str == "return" {
                cur = new_token(
                    TokenKind::TkReturn,
                    cur,
                    ident_str.to_string(),
                    input.len() - chars.clone().count(),
                );
                continue;
            }
            if ident_str == "if" {
                cur = new_token(
                    TokenKind::TkIf,
                    cur,
                    ident_str.to_string(),
                    input.len() - chars.clone().count(),
                );
                continue;
            }
            if ident_str == "else" {
                cur = new_token(
                    TokenKind::TkElse,
                    cur,
                    ident_str.to_string(),
                    input.len() - chars.clone().count(),
                );
                continue;
            }
            if ident_str == "while" {
                cur = new_token(
                    TokenKind::TkWhile,
                    cur,
                    ident_str.to_string(),
                    input.len() - chars.clone().count(),
                );
                continue;
            }
            if ident_str == "for" {
                cur = new_token(
                    TokenKind::TkFor,
                    cur,
                    ident_str.to_string(),
                    input.len() - chars.clone().count(),
                );
                continue;
            }
            if ident_str == "int" {
                cur = new_token(
                    TokenKind::TkInt,
                    cur,
                    ident_str.to_string(),
                    input.len() - chars.clone().count(),
                );
                continue;
            }
            if ident_str == "sizeof" {
                cur = new_token(
                    TokenKind::TkSizeof,
                    cur,
                    ident_str.to_string(),
                    input.len() - chars.clone().count(),
                );
                continue;
            }
            cur = new_token(
                TokenKind::TkIdent,
                cur,
                ident_str.to_string(),
                input.len() - chars.clone().count(),
            );
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
