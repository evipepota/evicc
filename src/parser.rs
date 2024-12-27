use std::borrow::BorrowMut;

use crate::tokenizer;

#[derive(Clone, Debug)]
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
    NdReturn, // Return
    NdIf,     // If
    NdElse,   // Else
    NdWhile,  // While
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

fn new_node_lvar(name: String, lvar: &mut Option<Box<tokenizer::LVar>>) -> Node {
    let tmp = tokenizer::find_lvar(lvar, &name);
    let offset = if let Some(offset) = tmp {
        offset
    } else {
        if let Some(lvar) = lvar {
            lvar.offset + 8
        } else {
            8
        }
    };

    *lvar = Some(Box::new(tokenizer::LVar::new(
        lvar.take(),
        name.clone(),
        offset,
    )));

    Node {
        kind: NodeKind::NdLvar,
        lhs: None,
        rhs: None,
        name,
        val: 0,
        offset,
    }
}

pub fn program(
    token: &mut Option<Box<tokenizer::Token>>,
    lvar: &mut Option<Box<tokenizer::LVar>>,
) -> Vec<Node> {
    let mut code = Vec::new();
    loop {
        let node = stmt(token, lvar);
        code.push(node.clone());
        if let Some(current) = token.borrow_mut() {
            if let tokenizer::TokenKind::TkEof = current.kind {
                return code;
            }
        }
    }
}

fn stmt(
    token: &mut Option<Box<tokenizer::Token>>,
    lvar: &mut Option<Box<tokenizer::LVar>>,
) -> Node {
    if tokenizer::consume_kind(tokenizer::TokenKind::TkReturn, &mut token.borrow_mut()) {
        let node = new_node(NodeKind::NdReturn, Some(Box::new(expr(token, lvar))), None);
        if tokenizer::consume(";", &mut token.borrow_mut()) {
            return node;
        } else {
            tokenizer::error_at(token.borrow_mut().as_ref().unwrap().loc, "expected ';'");
        }
    } else if tokenizer::consume_kind(tokenizer::TokenKind::TkIf, &mut token.borrow_mut()) {
        tokenizer::expect("(", &mut token.borrow_mut());
        let cond = expr(token, lvar);
        tokenizer::expect(")", &mut token.borrow_mut());
        let then = stmt(token, lvar);
        if tokenizer::consume_kind(tokenizer::TokenKind::TkElse, &mut token.borrow_mut()) {
            let els = stmt(token, lvar);
            return new_node(
                NodeKind::NdIf,
                Some(Box::new(cond)),
                Some(Box::new(new_node(
                    NodeKind::NdElse,
                    Some(Box::new(then)),
                    Some(Box::new(els)),
                ))),
            );
        }
        return new_node(NodeKind::NdIf, Some(Box::new(cond)), Some(Box::new(then)));
    } else if tokenizer::consume_kind(tokenizer::TokenKind::TkWhile, &mut token.borrow_mut()) {
        tokenizer::expect("(", &mut token.borrow_mut());
        let cond = expr(token, lvar);
        tokenizer::expect(")", &mut token.borrow_mut());
        let body = stmt(token, lvar);
        return new_node(
            NodeKind::NdWhile,
            Some(Box::new(cond)),
            Some(Box::new(body)),
        );
    }
    let node = expr(token, lvar);
    if tokenizer::consume(";", &mut token.borrow_mut()) {
        return node;
    } else {
        if let Some(current) = token.borrow_mut() {
            tokenizer::error_at(current.loc, "expected ';'");
        } else {
            tokenizer::error("unexpected error");
        }
    }
}

fn expr(
    token: &mut Option<Box<tokenizer::Token>>,
    lvar: &mut Option<Box<tokenizer::LVar>>,
) -> Node {
    return assign(token, lvar);
}

fn assign(
    token: &mut Option<Box<tokenizer::Token>>,
    lvar: &mut Option<Box<tokenizer::LVar>>,
) -> Node {
    let node = equality(token, lvar);
    if tokenizer::consume("=", &mut token.borrow_mut()) {
        return new_node(
            NodeKind::NdAssign,
            Some(Box::new(node)),
            Some(Box::new(assign(token, lvar))),
        );
    }
    return node;
}

fn equality(
    token: &mut Option<Box<tokenizer::Token>>,
    lvar: &mut Option<Box<tokenizer::LVar>>,
) -> Node {
    let mut node = relational(token, lvar);

    loop {
        if tokenizer::consume("==", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdEq,
                Some(Box::new(node)),
                Some(Box::new(relational(token, lvar))),
            );
        } else if tokenizer::consume("!=", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdNe,
                Some(Box::new(node)),
                Some(Box::new(relational(token, lvar))),
            );
        } else {
            return node;
        }
    }
}

fn relational(
    token: &mut Option<Box<tokenizer::Token>>,
    lvar: &mut Option<Box<tokenizer::LVar>>,
) -> Node {
    let mut node = add(token, lvar);

    loop {
        if tokenizer::consume("<", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdLt,
                Some(Box::new(node)),
                Some(Box::new(add(token, lvar))),
            );
        } else if tokenizer::consume("<=", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdLe,
                Some(Box::new(node)),
                Some(Box::new(add(token, lvar))),
            );
        } else if tokenizer::consume(">", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdGt,
                Some(Box::new(node)),
                Some(Box::new(add(token, lvar))),
            );
        } else if tokenizer::consume(">=", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdGe,
                Some(Box::new(node)),
                Some(Box::new(add(token, lvar))),
            );
        } else {
            return node;
        }
    }
}

fn add(token: &mut Option<Box<tokenizer::Token>>, lvar: &mut Option<Box<tokenizer::LVar>>) -> Node {
    let mut node = mul(token, lvar);

    loop {
        if tokenizer::consume("+", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdAdd,
                Some(Box::new(node)),
                Some(Box::new(mul(token, lvar))),
            );
        } else if tokenizer::consume("-", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdSub,
                Some(Box::new(node)),
                Some(Box::new(mul(token, lvar))),
            );
        } else {
            return node;
        }
    }
}

fn mul(token: &mut Option<Box<tokenizer::Token>>, lvar: &mut Option<Box<tokenizer::LVar>>) -> Node {
    let mut node = unary(token, lvar);

    loop {
        if tokenizer::consume("*", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdMul,
                Some(Box::new(node)),
                Some(Box::new(unary(token, lvar))),
            );
        } else if tokenizer::consume("/", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdDiv,
                Some(Box::new(node)),
                Some(Box::new(unary(token, lvar))),
            );
        } else {
            return node;
        }
    }
}

fn primary(
    token: &mut Option<Box<tokenizer::Token>>,
    lvar: &mut Option<Box<tokenizer::LVar>>,
) -> Node {
    if tokenizer::consume("(", &mut token.borrow_mut()) {
        let node = expr(token, lvar);
        tokenizer::expect(")", &mut token.borrow_mut());
        return node;
    }

    if let Some(current) = token {
        if let tokenizer::TokenKind::TkNum = current.kind {
            return new_node_num(tokenizer::expect_number(&mut token.borrow_mut()));
        } else if let tokenizer::TokenKind::TkIdent = current.kind {
            return new_node_lvar(tokenizer::expect_ident(&mut token.borrow_mut()), lvar);
        }
        tokenizer::error("expected number or ident");
    } else {
        tokenizer::error("unexpected error");
    }
}

fn unary(
    token: &mut Option<Box<tokenizer::Token>>,
    lvar: &mut Option<Box<tokenizer::LVar>>,
) -> Node {
    if tokenizer::consume("+", &mut token.borrow_mut()) {
        return primary(token, lvar);
    }
    if tokenizer::consume("-", &mut token.borrow_mut()) {
        return new_node(
            NodeKind::NdNeg,
            Some(Box::new(new_node_num(0))),
            Some(Box::new(primary(token, lvar))),
        );
    }
    primary(token, lvar)
}
