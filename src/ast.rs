use crate::lvar::LVar;
use crate::sema::Type;
use crate::sema::{new_type_int, new_type_ptr, TypeKind};
use crate::util::{error, find_lvar};

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
    pub var_type: Option<Box<Type>>,
    pub stmts: Vec<Node>,
}

pub fn new_node(kind: NodeKind, lhs: Option<Box<Node>>, rhs: Option<Box<Node>>) -> Node {
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

pub fn new_node_num(val: i32) -> Node {
    Node {
        kind: NodeKind::NdNum,
        lhs: None,
        rhs: None,
        name: String::new(),
        val,
        offset: 0,
        var_type: new_type_int(),
        stmts: Vec::new(),
    }
}

pub fn new_node_func(name: String, args: Vec<Node>) -> Node {
    let func_type = Type {
        ty: TypeKind::TyFunc,
        size: 8,
        ptr_to: new_type_int(),
    };
    Node {
        kind: NodeKind::NdFunc,
        lhs: None,
        rhs: None,
        name,
        val: 0,
        offset: 0,
        var_type: Some(Box::new(func_type)),
        stmts: args,
    }
}

pub fn new_node_lvar(name: String, lvar: &mut Option<Box<LVar>>) -> Node {
    let lvar = if let Some(lvar) = find_lvar(lvar, &name) {
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
        var_type: Some(Box::new(node_type)),
        stmts: Vec::new(),
    }
}

pub fn new_node_var_def(name: String, depth_pointer: usize, lvar: &mut Option<Box<LVar>>) -> Node {
    let offset = if let Some(_) = find_lvar(lvar, &name) {
        error("variable already declared");
    } else {
        if let Some(lvar) = lvar {
            lvar.offset + 8
        } else {
            8
        }
    };

    let mut node_type = new_type_int();
    for _ in 0..depth_pointer {
        node_type = new_type_ptr(node_type);
    }

    *lvar = Some(Box::new(LVar::new(
        lvar.take(),
        name.clone(),
        offset,
        node_type.clone().unwrap().as_ref().clone(),
    )));

    Node {
        kind: NodeKind::NdVardef,
        lhs: None,
        rhs: None,
        name,
        val: 0,
        offset,
        var_type: node_type,
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
