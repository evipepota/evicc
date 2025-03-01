use crate::ast::{Node, NodeKind};

#[derive(Clone, Debug, PartialEq)]
pub enum TypeKind {
    TyInt,
    TyPtr,
    TyArray,
}

impl TypeKind {
    pub fn size(&self) -> i32 {
        match self {
            TypeKind::TyInt => 4,
            TypeKind::TyPtr => 8,
            TypeKind::TyArray => 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Type {
    pub ty: TypeKind,
    pub size: usize,
    pub ptr_to: Option<Box<Type>>,
    pub array_size: usize,
}

pub fn new_type_int() -> Option<Box<Type>> {
    return new_type(TypeKind::TyInt, 4, None, 0);
}

pub fn new_type_ptr(node_type: Option<Box<Type>>) -> Option<Box<Type>> {
    return new_type(TypeKind::TyPtr, 8, node_type, 0);
}

pub fn new_type_array(node_type: Option<Box<Type>>, size: usize) -> Option<Box<Type>> {
    return new_type(
        TypeKind::TyArray,
        node_type.as_ref().unwrap().size * size,
        node_type,
        size,
    );
}

pub fn new_type(
    ty: TypeKind,
    size: usize,
    ptr_to: Option<Box<Type>>,
    array_size: usize,
) -> Option<Box<Type>> {
    Some(Box::new(Type {
        ty,
        size,
        ptr_to,
        array_size,
    }))
}

pub fn add_type(node: &mut Node) {
    if node.var_type.is_some() {
        return;
    }
    if let Some(ref mut lhs) = node.lhs {
        add_type(lhs);
    }
    if let Some(ref mut rhs) = node.rhs {
        add_type(rhs);
    }
    match node.kind {
        NodeKind::NdNum => node.var_type = new_type_int(),
        NodeKind::NdAdd | NodeKind::NdSub | NodeKind::NdMul | NodeKind::NdDiv => {
            node.var_type = node.lhs.as_ref().unwrap().var_type.clone()
        }
        NodeKind::NdAssign => node.var_type = node.lhs.as_ref().unwrap().var_type.clone(),
        NodeKind::NdEq
        | NodeKind::NdNe
        | NodeKind::NdLt
        | NodeKind::NdGt
        | NodeKind::NdGe
        | NodeKind::NdLe => node.var_type = new_type_int(),
        NodeKind::NdNeg => node.var_type = node.rhs.as_ref().unwrap().var_type.clone(),
        NodeKind::NdAddr => {
            if node.rhs.clone().unwrap().var_type.as_ref().unwrap().ty == TypeKind::TyArray {
                node.var_type = new_type_ptr(
                    node.rhs
                        .clone()
                        .unwrap()
                        .var_type
                        .clone()
                        .unwrap()
                        .ptr_to
                        .clone(),
                );
            } else {
                node.var_type = new_type_ptr(node.rhs.clone().unwrap().var_type.clone());
            }
        }
        NodeKind::NdDeref => {
            node.var_type = node
                .rhs
                .as_ref()
                .unwrap()
                .var_type
                .as_ref()
                .unwrap()
                .ptr_to
                .clone();
        }
        _ => {}
    }
}
