use std::{borrow::BorrowMut, iter};

use crate::ast::{
    new_node, new_node_block, new_node_func, new_node_gvar_def, new_node_gvar_def_array,
    new_node_lvar, new_node_num, new_node_var_def, new_node_var_def_array, Node, NodeKind,
};
use crate::lvar::LVar;
use crate::sema::{add_type, TypeKind};
use crate::tokenizer;
use crate::util::{
    check, consume, consume_kind, error, error_at, expect, expect_ident, expect_number,
};

/*
program = (global | function)*
*/
pub fn program(
    token: &mut Option<Box<tokenizer::Token>>,
) -> (Vec<(Vec<Node>, Vec<Node>, i32, String)>, Option<Box<LVar>>) {
    let mut code = Vec::new();
    let mut gloval_vars = None;
    while let Some(current) = token {
        match current.kind {
            tokenizer::TokenKind::TkInt => {
                global_or_function(token, &mut code, &mut gloval_vars);
            }
            tokenizer::TokenKind::TkEof => break,
            _ => {
                error_at(current.loc, "expected function");
            }
        }
    }
    return (code, gloval_vars);
}

// global_or_function = "int" "*"? ident (function | global)
fn global_or_function(
    token: &mut Option<Box<tokenizer::Token>>,
    code: &mut Vec<(Vec<Node>, Vec<Node>, i32, String)>,
    gvar: &mut Option<Box<LVar>>,
) {
    if !consume_kind(tokenizer::TokenKind::TkInt, token) {
        error_at(token.as_ref().unwrap().loc, "expected 'int'");
    }
    let depth_pointer = iter::repeat(()).take_while(|_| consume("*", token)).count();
    let ident = expect_ident(token);
    if check("(", token) {
        let func = function(token, ident, gvar);
        code.push(func);
    } else {
        global(token, ident, depth_pointer, gvar);
    }
}

/*
global =  ("[" num "]")? ";"
*/
fn global(
    token: &mut Option<Box<tokenizer::Token>>,
    ident: String,
    depth_pointer: usize,
    gvar: &mut Option<Box<LVar>>,
) -> Node {
    if consume(";", token) {
        return new_node_gvar_def(ident, depth_pointer, gvar);
    } else if consume("[", token) {
        let size = expect_number(token);
        expect("]", token);
        expect(";", token);
        return new_node_gvar_def_array(ident, size, gvar, TypeKind::TyInt);
    } else {
        error_at(token.as_ref().unwrap().loc, "expected ';'");
    }
}

/*
function = "(" ("int" "*"? ident ("," "int" "*"? ident)*)? ")" "{" stmt* "}"
*/
fn function(
    token: &mut Option<Box<tokenizer::Token>>,
    ident: String,
    gvar: &mut Option<Box<LVar>>,
) -> (Vec<Node>, Vec<Node>, i32, String) {
    expect("(", token);
    let mut lvar = None;
    let mut args = Vec::new();
    if !consume(")", token) {
        if !consume_kind(tokenizer::TokenKind::TkInt, token) {
            error_at(token.as_ref().unwrap().loc, "expected 'int'");
        }
        let depth_pointer = iter::repeat(()).take_while(|_| consume("*", token)).count();
        args.push(new_node_var_def(
            expect_ident(token),
            depth_pointer,
            &mut lvar,
        ));
        while consume(",", token) {
            if !consume_kind(tokenizer::TokenKind::TkInt, token) {
                error_at(token.as_ref().unwrap().loc, "expected 'int'");
            }
            let depth_pointer = iter::repeat(()).take_while(|_| consume("*", token)).count();
            args.push(new_node_var_def(
                expect_ident(token),
                depth_pointer,
                &mut lvar,
            ));
        }
        expect(")", token);
    }
    expect("{", token);
    let mut stmts = Vec::new();
    while !consume("}", token) {
        let node = stmt(token, &mut lvar, gvar);
        stmts.push(node.clone());
    }
    if let Some(lvar) = lvar {
        return (args, stmts, lvar.offset + 8, ident);
    } else {
        return (args, stmts, 0, ident);
    }
}

/*
stmt = expr ";"
     | "int" "*"? ident ";"
     | "int" ident "[" num "]" ";"
     | "return" expr ";"
     | "if" "(" expr ")" stmt ("else" stmt)?
     | "while" "(" expr ")" stmt
     | "for" "(" expr? ";" expr? ";" expr? ")" stmt
     | "{" stmt* "}"
*/
fn stmt(
    token: &mut Option<Box<tokenizer::Token>>,
    lvar: &mut Option<Box<LVar>>,
    gvar: &mut Option<Box<LVar>>,
) -> Node {
    if consume_kind(tokenizer::TokenKind::TkReturn, &mut token.borrow_mut()) {
        let node = new_node(
            NodeKind::NdReturn,
            Some(Box::new(expr(token, lvar, gvar))),
            None,
        );
        if consume(";", &mut token.borrow_mut()) {
            return node;
        } else {
            error_at(token.borrow_mut().as_ref().unwrap().loc, "expected ';'");
        }
    } else if consume_kind(tokenizer::TokenKind::TkIf, &mut token.borrow_mut()) {
        expect("(", &mut token.borrow_mut());
        let cond = expr(token, lvar, gvar);
        expect(")", &mut token.borrow_mut());
        let then = stmt(token, lvar, gvar);
        if consume_kind(tokenizer::TokenKind::TkElse, &mut token.borrow_mut()) {
            let els = stmt(token, lvar, gvar);
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
    } else if consume_kind(tokenizer::TokenKind::TkWhile, &mut token.borrow_mut()) {
        expect("(", &mut token.borrow_mut());
        let cond = expr(token, lvar, gvar);
        expect(")", &mut token.borrow_mut());
        let body = stmt(token, lvar, gvar);
        return new_node(
            NodeKind::NdWhile,
            Some(Box::new(cond)),
            Some(Box::new(body)),
        );
    } else if consume_kind(tokenizer::TokenKind::TkFor, &mut token.borrow_mut()) {
        expect("(", &mut token.borrow_mut());
        let init = if consume(";", token) {
            None
        } else {
            let result = expr(token, lvar, gvar);
            expect(";", &mut token.borrow_mut());
            Some(result)
        };
        let cond = if consume(";", token) {
            None
        } else {
            let result = expr(token, lvar, gvar);
            expect(";", &mut token.borrow_mut());
            Some(result)
        };
        let inc = if consume(")", token) {
            None
        } else {
            let result = expr(token, lvar, gvar);
            expect(")", &mut token.borrow_mut());
            Some(result)
        };
        let body = stmt(token, lvar, gvar);
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
    } else if consume_kind(tokenizer::TokenKind::TkInt, &mut token.borrow_mut()) {
        let depth_pointer = iter::repeat(()).take_while(|_| consume("*", token)).count();
        let ident = expect_ident(token);
        if consume(";", token) {
            return new_node_var_def(ident, depth_pointer, lvar);
        } else if consume("[", token) {
            assert!(depth_pointer == 0);
            let size = expect_number(token);
            expect("]", token);
            expect(";", token);
            return new_node_var_def_array(ident, size, lvar, TypeKind::TyInt);
        } else {
            error_at(token.as_ref().unwrap().loc, "expected ';'");
        }
    } else if consume("{", token) {
        let mut stmts = Vec::new();
        while !consume("}", token) {
            stmts.push(stmt(token, lvar, gvar));
        }
        return new_node_block(stmts);
    }
    let node = expr(token, lvar, gvar);
    if consume(";", &mut token.borrow_mut()) {
        return node;
    } else {
        if let Some(current) = token.borrow_mut() {
            error_at(current.loc, "expected ';'");
        } else {
            error("unexpected error");
        }
    }
}

/*
expr = assign
*/
fn expr(
    token: &mut Option<Box<tokenizer::Token>>,
    lvar: &mut Option<Box<LVar>>,
    gvar: &mut Option<Box<LVar>>,
) -> Node {
    return assign(token, lvar, gvar);
}

/*
assign = equality ("=" assign)?
*/
fn assign(
    token: &mut Option<Box<tokenizer::Token>>,
    lvar: &mut Option<Box<LVar>>,
    gvar: &mut Option<Box<LVar>>,
) -> Node {
    let node = equality(token, lvar, gvar);
    if consume("=", &mut token.borrow_mut()) {
        return new_node(
            NodeKind::NdAssign,
            Some(Box::new(node)),
            Some(Box::new(assign(token, lvar, gvar))),
        );
    }
    return node;
}

/*
equality = relational ("==" relational | "!=" relational)*
*/
fn equality(
    token: &mut Option<Box<tokenizer::Token>>,
    lvar: &mut Option<Box<LVar>>,
    gvar: &mut Option<Box<LVar>>,
) -> Node {
    let mut node = relational(token, lvar, gvar);

    loop {
        if consume("==", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdEq,
                Some(Box::new(node)),
                Some(Box::new(relational(token, lvar, gvar))),
            );
        } else if consume("!=", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdNe,
                Some(Box::new(node)),
                Some(Box::new(relational(token, lvar, gvar))),
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
    lvar: &mut Option<Box<LVar>>,
    gvar: &mut Option<Box<LVar>>,
) -> Node {
    let mut node = add(token, lvar, gvar);

    loop {
        if consume("<", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdLt,
                Some(Box::new(node)),
                Some(Box::new(add(token, lvar, gvar))),
            );
        } else if consume("<=", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdLe,
                Some(Box::new(node)),
                Some(Box::new(add(token, lvar, gvar))),
            );
        } else if consume(">", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdGt,
                Some(Box::new(node)),
                Some(Box::new(add(token, lvar, gvar))),
            );
        } else if consume(">=", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdGe,
                Some(Box::new(node)),
                Some(Box::new(add(token, lvar, gvar))),
            );
        } else {
            return node;
        }
    }
}

/*
add = mul ("+" mul | "-" mul)*
*/
fn add(
    token: &mut Option<Box<tokenizer::Token>>,
    lvar: &mut Option<Box<LVar>>,
    gvar: &mut Option<Box<LVar>>,
) -> Node {
    let mut node = mul(token, lvar, gvar);

    loop {
        if consume("+", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdAdd,
                Some(Box::new(node)),
                Some(Box::new(mul(token, lvar, gvar))),
            );
            add_type(&mut node);
        } else if consume("-", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdSub,
                Some(Box::new(node)),
                Some(Box::new(mul(token, lvar, gvar))),
            );
            add_type(&mut node);
        } else {
            return node;
        }
    }
}

/*
mul = unary ("*" unary | "/" unary)*
*/
fn mul(
    token: &mut Option<Box<tokenizer::Token>>,
    lvar: &mut Option<Box<LVar>>,
    gvar: &mut Option<Box<LVar>>,
) -> Node {
    let mut node = unary(token, lvar, gvar);

    loop {
        if consume("*", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdMul,
                Some(Box::new(node)),
                Some(Box::new(unary(token, lvar, gvar))),
            );
        } else if consume("/", &mut token.borrow_mut()) {
            node = new_node(
                NodeKind::NdDiv,
                Some(Box::new(node)),
                Some(Box::new(unary(token, lvar, gvar))),
            );
        } else {
            return node;
        }
    }
}

/*
unary = "sizeof" unary | ("+" | "-")? primary | "*" unary | "&" unary
*/
fn unary(
    token: &mut Option<Box<tokenizer::Token>>,
    lvar: &mut Option<Box<LVar>>,
    gvar: &mut Option<Box<LVar>>,
) -> Node {
    if consume("+", &mut token.borrow_mut()) {
        return primary(token, lvar, gvar);
    }
    if consume("-", &mut token.borrow_mut()) {
        return new_node(
            NodeKind::NdNeg,
            Some(Box::new(new_node_num(0))),
            Some(Box::new(primary(token, lvar, gvar))),
        );
    }
    if consume_kind(tokenizer::TokenKind::TkSizeof, token) {
        let mut node = unary(token, lvar, gvar);
        add_type(&mut node);
        if let Some(ty) = node.var_type {
            return new_node_num(ty.size as i32);
        } else {
            error("no type");
        }
    }
    if consume("*", &mut token.borrow_mut()) {
        let mut new_deref_node = new_node(
            NodeKind::NdDeref,
            None,
            Some(Box::new(unary(token, lvar, gvar))),
        );
        add_type(&mut new_deref_node);
        return new_deref_node;
    }
    if consume("&", &mut token.borrow_mut()) {
        return new_node(
            NodeKind::NdAddr,
            None,
            Some(Box::new(unary(token, lvar, gvar))),
        );
    }
    primary(token, lvar, gvar)
}

/*
primary = num | ident | ident "(" expr ")" | ident "[" expr "]"
*/
fn primary(
    token: &mut Option<Box<tokenizer::Token>>,
    lvar: &mut Option<Box<LVar>>,
    gvar: &mut Option<Box<LVar>>,
) -> Node {
    if consume("(", &mut token.borrow_mut()) {
        let node = expr(token, lvar, gvar);
        expect(")", &mut token.borrow_mut());
        return node;
    }

    if let Some(current) = token {
        if let tokenizer::TokenKind::TkNum = current.kind {
            return new_node_num(expect_number(&mut token.borrow_mut()));
        } else if let tokenizer::TokenKind::TkIdent = current.kind {
            let ident = expect_ident(&mut token.borrow_mut());

            if consume("(", &mut token.borrow_mut()) {
                let mut args = Vec::new();
                if !consume(")", &mut token.borrow_mut()) {
                    args.push(expr(token, lvar, gvar));
                    while consume(",", &mut token.borrow_mut()) {
                        args.push(expr(token, lvar, gvar));
                    }
                    expect(")", &mut token.borrow_mut());
                }
                return new_node_func(ident, args);
            } else if consume("[", &mut token.borrow_mut()) {
                let node = expr(token, lvar, gvar);
                expect("]", &mut token.borrow_mut());
                let array_node = new_node_lvar(ident, lvar, gvar);
                let add_node = new_node(
                    NodeKind::NdAdd,
                    Some(Box::new(array_node)),
                    Some(Box::new(node)),
                );
                return new_node(NodeKind::NdDeref, None, Some(Box::new(add_node)));
            }

            return new_node_lvar(ident, lvar, gvar);
        }
        error("expected number or ident");
    } else {
        error("unexpected error");
    }
}
