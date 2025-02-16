use crate::parser::{Node, NodeKind};

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
