use crate::parser::{Node, NodeKind};
use crate::tokenizer::{Type, TypeKind};

pub fn new_type_int() -> Option<Box<Type>> {
    return new_type(TypeKind::TyInt, 4, None);
}

pub fn new_type_ptr(node_type: Option<Box<Type>>) -> Option<Box<Type>> {
    return new_type(TypeKind::TyPtr, 8, node_type);
}

pub fn new_type(ty: TypeKind, size: usize, ptr_to: Option<Box<Type>>) -> Option<Box<Type>> {
    Some(Box::new(Type { ty, size, ptr_to }))
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
        NodeKind::NdAddr => node.var_type = new_type_ptr(node.rhs.clone().unwrap().var_type),
        NodeKind::NdDeref => {
            node.var_type = node
                .rhs
                .as_ref()
                .unwrap()
                .var_type
                .as_ref()
                .unwrap()
                .ptr_to
                .clone()
        }
        _ => {}
    }
}
