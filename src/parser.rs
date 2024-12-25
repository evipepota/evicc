use std::borrow::BorrowMut;

use crate::tokenizer;

#[derive(Clone)]
pub enum NodeKind {
    NdAdd,    // +
    NdSub,    // -
    NdMul,    // *
    NdDiv,    // /
    NdNeg,    // unary -
    NdEq,     // ==
    NdNe,     // !=
    NdGt,     // >
    NdGe,     // >=
    NdLt,     // <
    NdLe,     // <=
    NdAssign, // =
    NdNum,    // Integer
    NdLvar,   // Local variable
}

#[derive(Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
    pub name: String,
    pub val: i32,
    pub offset: i32,
}

fn new_node(kind: NodeKind, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>) -> Node {
    Node {
        kind,
        lhs,
        rhs,
        name: String::new(),
        val: 0,
        offset: 0,
    }
}

fn new_node_num(val: i32) -> Node {
    Node {
        kind: NodeKind::NdNum,
        lhs: None,
        rhs: None,
        name: String::new(),
        val,
        offset: 0,
    }
}

fn new_node_lvar(name: String, offset: i32) -> Node {
    Node {
        kind: NodeKind::NdLvar,
        lhs: None,
        rhs: None,
        name,
        val: 0,
        offset,
    }
}

pub fn program(token: &mut Option<Box<tokenizer::Token>>) -> Vec<Node> {
    let mut code = Vec::new();
    loop {
        let node = stmt(token);
        code.push(node.clone());
        if let Some(current) = token.borrow_mut() {
            if let tokenizer::TokenKind::TkEof = current.kind {
                return code;
            }
        }
    }
}

fn stmt(token: &mut Option<Box<tokenizer::Token>>) -> Node {
    let node = expr(token);
    if tokenizer::consume(";", &mut token.borrow_mut()) {
        return node;
    } else {
        tokenizer::error_at(0, "expected ';'");
    }
}

fn expr(token: &mut Option<Box<tokenizer::Token>>) -> Node {
    return assign(token);
}

fn assign(token: &mut Option<Box<tokenizer::Token>>) -> Node {
    let node = equality(token);
    if tokenizer::consume("=", &mut token.borrow_mut()) {
        return new_node(
            NodeKind::NdAssign,
            Some(Box::new(node)),
            Some(Box::new(assign(token))),
        );
    }
    return node;
}

fn equality(token: &mut Option<Box<tokenizer::Token>>) -> Node {
    let mut node = relational(token);

    loop {
        if tokenizer::consume("==", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdEq,
                Some(Box::new(node)),
                Some(Box::new(relational(token))),
            );
        } else if tokenizer::consume("!=", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdNe,
                Some(Box::new(node)),
                Some(Box::new(relational(token))),
            );
        } else {
            return node;
        }
    }
}

fn relational(token: &mut Option<Box<tokenizer::Token>>) -> Node {
    let mut node = add(token);

    loop {
        if tokenizer::consume("<", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdLt,
                Some(Box::new(node)),
                Some(Box::new(add(token))),
            );
        } else if tokenizer::consume("<=", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdLe,
                Some(Box::new(node)),
                Some(Box::new(add(token))),
            );
        } else if tokenizer::consume(">", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdGt,
                Some(Box::new(node)),
                Some(Box::new(add(token))),
            );
        } else if tokenizer::consume(">=", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdGe,
                Some(Box::new(node)),
                Some(Box::new(add(token))),
            );
        } else {
            return node;
        }
    }
}

fn add(token: &mut Option<Box<tokenizer::Token>>) -> Node {
    let mut node = mul(token);

    loop {
        if tokenizer::consume("+", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdAdd,
                Some(Box::new(node)),
                Some(Box::new(mul(token))),
            );
        } else if tokenizer::consume("-", &mut token.borrow_mut()) {
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
        if tokenizer::consume("*", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdMul,
                Some(Box::new(node)),
                Some(Box::new(unary(token))),
            );
        } else if tokenizer::consume("/", &mut token.borrow_mut()) {
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
    if tokenizer::consume("(", &mut token.borrow_mut()) {
        let node = expr(token);
        tokenizer::expect(")", &mut token.borrow_mut());
        return node;
    }

    if let Some(current) = token {
        if let tokenizer::TokenKind::TkNum = current.kind {
            return new_node_num(tokenizer::expect_number(&mut token.borrow_mut()));
        } else if let tokenizer::TokenKind::TkIdent = current.kind {
            let offset = (current.str.chars().nth(0).unwrap() as i32 - 'a' as i32 + 1) * 8;
            return new_node_lvar(tokenizer::expect_ident(&mut token.borrow_mut()), offset);
        }
        tokenizer::error("expected number or ident");
    } else {
        tokenizer::error("unexpected error");
    }
}

fn unary(token: &mut Option<Box<tokenizer::Token>>) -> Node {
    if tokenizer::consume("+", &mut token.borrow_mut()) {
        return primary(token);
    }
    if tokenizer::consume("-", &mut token.borrow_mut()) {
        return new_node(
            NodeKind::NdNeg,
            Some(Box::new(new_node_num(0))),
            Some(Box::new(primary(token))),
        );
    }
    primary(token)
}
