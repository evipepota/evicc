use std::{borrow::BorrowMut, iter};

use crate::tokenizer::{self, consume, error, error_at};

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
    NdDeref,  // *
    NdAddr,   // &
    NdNum,    // Integer
    NdLvar,   // Local variable
    NdReturn, // Return
    NdIf,     // If
    NdElse,   // Else
    NdWhile,  // While
    NdFor,    // For
    NdBlock,  // Block
    NdFunc,   // Function
    NdVardef, // Variable definition
}

#[derive(Clone)]
pub struct Node {
    pub kind: NodeKind,
    pub lhs: Option<Box<Node>>,
    pub rhs: Option<Box<Node>>,
    pub name: String,
    pub val: i32,
    pub offset: i32,
    pub var_type: Option<tokenizer::Type>,
    pub stmts: Vec<Node>,
}

fn new_node(kind: NodeKind, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>) -> Node {
    Node {
        kind,
        lhs,
        rhs,
        name: String::new(),
        val: 0,
        offset: 0,
        var_type: None,
        stmts: Vec::new(),
    }
}

fn new_node_num(val: i32) -> Node {
    let num_type = tokenizer::Type {
        ty: tokenizer::TypeKind::TyInt,
        ptr_to: None,
    };
    Node {
        kind: NodeKind::NdNum,
        lhs: None,
        rhs: None,
        name: String::new(),
        val,
        offset: 0,
        var_type: Some(num_type),
        stmts: Vec::new(),
    }
}

fn new_node_func(name: String, args: Vec<Node>) -> Node {
    let func_type = tokenizer::Type {
        ty: tokenizer::TypeKind::TyFunc,
        ptr_to: Some(Box::new(tokenizer::Type {
            ty: tokenizer::TypeKind::TyInt,
            ptr_to: None,
        })),
    };
    Node {
        kind: NodeKind::NdFunc,
        lhs: None,
        rhs: None,
        name,
        val: 0,
        offset: 0,
        var_type: Some(func_type),
        stmts: args,
    }
}

fn new_node_lvar(name: String, lvar: &mut Option<Box<tokenizer::LVar>>) -> Node {
    let lvar = if let Some(lvar) = tokenizer::find_lvar(lvar, &name) {
        *lvar
    } else {
        println!("{}", name);
        error("not declared variable");
    };

    let node_type = lvar.ty.clone();

    Node {
        kind: NodeKind::NdLvar,
        lhs: None,
        rhs: None,
        name,
        val: 0,
        offset: lvar.offset,
        var_type: Some(node_type),
        stmts: Vec::new(),
    }
}

fn new_node_var_def(
    name: String,
    depth_pointer: usize,
    lvar: &mut Option<Box<tokenizer::LVar>>,
) -> Node {
    let offset = if let Some(_) = tokenizer::find_lvar(lvar, &name) {
        error("variable already declared");
    } else {
        if let Some(lvar) = lvar {
            lvar.offset + 8
        } else {
            8
        }
    };

    let mut node_type = tokenizer::Type {
        ty: tokenizer::TypeKind::TyInt,
        ptr_to: None,
    };
    for _ in 0..depth_pointer {
        node_type = tokenizer::Type {
            ty: tokenizer::TypeKind::TyPtr,
            ptr_to: Some(Box::new(node_type)),
        };
    }

    *lvar = Some(Box::new(tokenizer::LVar::new(
        lvar.take(),
        name.clone(),
        offset,
        node_type.clone(),
    )));

    Node {
        kind: NodeKind::NdVardef,
        lhs: None,
        rhs: None,
        name,
        val: 0,
        offset,
        var_type: Some(node_type),
        stmts: Vec::new(),
    }
}

pub fn new_node_block(stmts: Vec<Node>) -> Node {
    Node {
        kind: NodeKind::NdBlock,
        lhs: None,
        rhs: None,
        name: String::new(),
        val: 0,
        offset: 0,
        var_type: None,
        stmts,
    }
}

/*
program = function*
*/
pub fn program(
    token: &mut Option<Box<tokenizer::Token>>,
) -> Vec<(Vec<Node>, Vec<Node>, i32, String)> {
    let mut code = Vec::new();
    while let Some(current) = token {
        match current.kind {
            tokenizer::TokenKind::TkInt => {
                code.push(function(token));
            }
            tokenizer::TokenKind::TkEof => break,
            _ => {
                tokenizer::error_at(current.loc, "expected function");
            }
        }
    }
    return code;
}

/*
function = "int" ident "(" ("int" "*"? ident ("," "int" "*"? ident)*)? ")" "{" stmt* "}"
*/
fn function(token: &mut Option<Box<tokenizer::Token>>) -> (Vec<Node>, Vec<Node>, i32, String) {
    if !tokenizer::consume_kind(tokenizer::TokenKind::TkInt, token) {
        error_at(token.as_ref().unwrap().loc, "expected 'int'");
    }
    let name = tokenizer::expect_ident(token);
    tokenizer::expect("(", token);
    let mut lvar = None;
    let mut args = Vec::new();
    if !tokenizer::consume(")", token) {
        if !tokenizer::consume_kind(tokenizer::TokenKind::TkInt, token) {
            error_at(token.as_ref().unwrap().loc, "expected 'int'");
        }
        let depth_pointer = iter::repeat(())
            .take_while(|_| tokenizer::consume("*", token))
            .count();
        args.push(new_node_var_def(
            tokenizer::expect_ident(token),
            depth_pointer,
            &mut lvar,
        ));
        while tokenizer::consume(",", token) {
            if !tokenizer::consume_kind(tokenizer::TokenKind::TkInt, token) {
                error_at(token.as_ref().unwrap().loc, "expected 'int'");
            }
            let depth_pointer = iter::repeat(())
                .take_while(|_| tokenizer::consume("*", token))
                .count();
            args.push(new_node_var_def(
                tokenizer::expect_ident(token),
                depth_pointer,
                &mut lvar,
            ));
        }
        tokenizer::expect(")", token);
    }
    tokenizer::expect("{", token);
    let mut stmts = Vec::new();
    while !tokenizer::consume("}", token) {
        let node = stmt(token, &mut lvar);
        stmts.push(node.clone());
    }
    if let Some(lvar) = lvar {
        return (args, stmts, lvar.offset + 8, name);
    } else {
        return (args, stmts, 0, name);
    }
}

/*
stmt = expr ";"
     | "int" "*"? expr ";"
     | "return" expr ";"
     | "if" "(" expr ")" stmt ("else" stmt)?
     | "while" "(" expr ")" stmt
     | "for" "(" expr? ";" expr? ";" expr? ")" stmt
     | "{" stmt* "}"
*/
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
    } else if tokenizer::consume_kind(tokenizer::TokenKind::TkFor, &mut token.borrow_mut()) {
        tokenizer::expect("(", &mut token.borrow_mut());
        let init = if tokenizer::consume(";", token) {
            None
        } else {
            let result = expr(token, lvar);
            tokenizer::expect(";", &mut token.borrow_mut());
            Some(result)
        };
        let cond = if tokenizer::consume(";", token) {
            None
        } else {
            let result = expr(token, lvar);
            tokenizer::expect(";", &mut token.borrow_mut());
            Some(result)
        };
        let inc = if tokenizer::consume(")", token) {
            None
        } else {
            let result = expr(token, lvar);
            tokenizer::expect(")", &mut token.borrow_mut());
            Some(result)
        };
        let body = stmt(token, lvar);
        return new_node(
            NodeKind::NdFor,
            init.map(Box::new),
            Some(Box::new(new_node(
                NodeKind::NdFor,
                cond.map(Box::new),
                Some(Box::new(new_node(
                    NodeKind::NdFor,
                    inc.map(Box::new),
                    Some(Box::new(body)),
                ))),
            ))),
        );
    } else if tokenizer::consume_kind(tokenizer::TokenKind::TkInt, &mut token.borrow_mut()) {
        let depth_pointer = iter::repeat(())
            .take_while(|_| tokenizer::consume("*", token))
            .count();
        let ident = tokenizer::expect_ident(token);
        if tokenizer::consume(";", token) {
            return new_node_var_def(ident, depth_pointer, lvar);
        } else {
            tokenizer::error_at(token.as_ref().unwrap().loc, "expected ';'");
        }
    } else if consume("{", token) {
        let mut stmts = Vec::new();
        while !consume("}", token) {
            stmts.push(stmt(token, lvar));
        }
        return new_node_block(stmts);
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

/*
expr = assign
*/
fn expr(
    token: &mut Option<Box<tokenizer::Token>>,
    lvar: &mut Option<Box<tokenizer::LVar>>,
) -> Node {
    return assign(token, lvar);
}

/*
assign = equality ("=" assign)?
*/
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

/*
equality = relational ("==" relational | "!=" relational)*
*/
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

/*
relational = add ("<" add | "<=" add | ">" add | ">=" add)*
*/
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

/*
add = mul ("+" mul | "-" mul)*
*/
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

/*
mul = unary ("*" unary | "/" unary)*
*/
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

/*
primary = num | ident | ident "(" expr ")"
*/
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
            let ident = tokenizer::expect_ident(&mut token.borrow_mut());

            if tokenizer::consume("(", &mut token.borrow_mut()) {
                let mut args = Vec::new();
                if !tokenizer::consume(")", &mut token.borrow_mut()) {
                    args.push(expr(token, lvar));
                    while tokenizer::consume(",", &mut token.borrow_mut()) {
                        args.push(expr(token, lvar));
                    }
                    tokenizer::expect(")", &mut token.borrow_mut());
                }
                return new_node_func(ident, args);
            }

            return new_node_lvar(ident, lvar);
        }
        tokenizer::error("expected number or ident");
    } else {
        tokenizer::error("unexpected error");
    }
}

/*
unary = ("+" | "-")? primary | "*" unary | "&" unary
*/
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
    if tokenizer::consume("*", &mut token.borrow_mut()) {
        return new_node(NodeKind::NdDeref, None, Some(Box::new(unary(token, lvar))));
    }
    if tokenizer::consume("&", &mut token.borrow_mut()) {
        return new_node(NodeKind::NdAddr, None, Some(Box::new(unary(token, lvar))));
    }
    primary(token, lvar)
}
