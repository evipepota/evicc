use std::borrow::BorrowMut;

use crate::tokenizer;

enum NodeKind {
    NdAdd, // +
    NdSub, // -
    NdMul, // *
    NdDiv, // /
    NdNum, // Integer
}

pub struct Node {
    kind: NodeKind,
    lhs: Option<Box<Node>>,
    rhs: Option<Box<Node>>,
    val: i32,
}

fn new_node(kind: NodeKind, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>) -> Node {
    Node {
        kind,
        lhs,
        rhs,
        val: 0,
    }
}

fn new_node_num(val: i32) -> Node {
    Node {
        kind: NodeKind::NdNum,
        lhs: None,
        rhs: None,
        val,
    }
}

pub fn expr(token: &mut Option<Box<tokenizer::Token>>) -> Node {
    let mut node = mul(token);

    loop {
        if tokenizer::consume('+', &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdAdd,
                Some(Box::new(node)),
                Some(Box::new(mul(token))),
            );
        } else if tokenizer::consume('-', &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdSub,
                Some(Box::new(node)),
                Some(Box::new(mul(token))),
            );
        } else {
            return node;
        }
    }
}

fn mul(token: &mut Option<Box<tokenizer::Token>>) -> Node {
    let mut node = unary(token);

    loop {
        if tokenizer::consume('*', &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdMul,
                Some(Box::new(node)),
                Some(Box::new(unary(token))),
            );
        } else if tokenizer::consume('/', &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdDiv,
                Some(Box::new(node)),
                Some(Box::new(unary(token))),
            );
        } else {
            return node;
        }
    }
}

fn primary(token: &mut Option<Box<tokenizer::Token>>) -> Node {
    if tokenizer::consume('(', &mut token.borrow_mut()) {
        let node = expr(token);
        tokenizer::expect(')', &mut token.borrow_mut());
        return node;
    }

    if let Some(current) = token {
        if let tokenizer::TokenKind::TkNum = current.kind {
            return new_node_num(tokenizer::expect_number(&mut token.borrow_mut()));
        }
        tokenizer::error_at(current.loc, "expected number");
    } else {
        tokenizer::error("unexpected error");
    }
}

fn unary(token: &mut Option<Box<tokenizer::Token>>) -> Node {
    if tokenizer::consume('+', &mut token.borrow_mut()) {
        return primary(token);
    }
    if tokenizer::consume('-', &mut token.borrow_mut()) {
        return new_node(
            NodeKind::NdSub,
            Some(Box::new(new_node_num(0))),
            Some(Box::new(primary(token))),
        );
    }
    primary(token)
}

pub fn gen(node: Node) {
    if let NodeKind::NdNum = node.kind {
        println!("  push {}", node.val);
        return;
    }

    if let Some(lhs) = node.lhs {
        gen(*lhs);
    }
    if let Some(rhs) = node.rhs {
        gen(*rhs);
    }

    println!("  pop rdi");
    println!("  pop rax");

    match node.kind {
        NodeKind::NdAdd => {
            println!("  add rax, rdi");
        }
        NodeKind::NdSub => {
            println!("  sub rax, rdi");
        }
        NodeKind::NdMul => {
            println!("  imul rax, rdi");
        }
        NodeKind::NdDiv => {
            println!("  cqo");
            println!("  idiv rdi");
        }
        _ => {}
    }

    println!("  push rax");
}
