use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};
use lazy_static::lazy_static;
use std::sync::RwLock;

use crate::parser::{Node, NodeKind};
use crate::tokenizer::{LVar, Token, TokenKind};

lazy_static! {
    pub static ref USER_INPUT: RwLock<Option<String>> = RwLock::new(None);
}

pub fn error(msg: &str) -> ! {
    eprintln!("Error: {}", msg);
    process::exit(1);
}

pub fn error_at(loc: usize, msg: &str) -> ! {
    if let Some(ref user_input) = *USER_INPUT.write().unwrap() {
        let input = &user_input;
        eprintln!("{}", input);
        eprintln!("{:>width$}^ {}", "", msg, width = loc);
    } else {
        eprintln!("Error: {}", msg);
    }
    process::exit(1);
}

pub fn consume(op: &str, token: &mut Option<Box<Token>>) -> bool {
    if let Some(current) = token {
        if let TokenKind::TkReserved = current.kind {
            if current.str == op {
                *token = current.next.take();
                return true;
            }
        }
    }
    false
}

pub fn consume_kind(kind: TokenKind, token: &mut Option<Box<Token>>) -> bool {
    if let Some(current) = token {
        if kind == current.kind {
            *token = current.next.take();
            return true;
        }
    }
    false
}

pub fn expect(op: &str, token: &mut Option<Box<Token>>) {
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

pub fn expect_ident(token: &mut Option<Box<Token>>) -> String {
    if let Some(current) = token {
        if let TokenKind::TkIdent = current.kind {
            let val = current.str.clone();
            *token = current.next.take();
            return val;
        }
        error_at(current.loc, "expected ident");
    } else {
        error("unexpected error");
    }
}

pub fn find_lvar(lvar: &Option<Box<LVar>>, name: &str) -> Option<Box<LVar>> {
    if let Some(current) = lvar {
        if current.name == name {
            return Some(current.clone());
        }
        find_lvar(&current.next, name)
    } else {
        None
    }
}

pub fn gen_label() -> usize {
    static LABEL: AtomicUsize = AtomicUsize::new(0);
    LABEL.fetch_add(1, Ordering::Relaxed)
}

#[allow(dead_code)]
pub fn at_eof(token: &Option<Box<Token>>) -> bool {
    if let Some(token) = token {
        matches!(token.kind, TokenKind::TkEof)
    } else {
        false
    }
}

pub fn calculate_pointer_depth(mut node: Option<Box<Node>>) -> (i32, Option<Box<Node>>) {
    let mut ptr_depth = 0;

    while let Some(rhs) = node.clone() {
        match rhs.kind {
            NodeKind::NdDeref => ptr_depth += 1,
            NodeKind::NdAddr => ptr_depth -= 1,
            NodeKind::NdLvar => break,
            _ => {}
        }
        node = rhs.rhs;
    }

    (ptr_depth, node)
}
