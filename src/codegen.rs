use crate::parser::{Node, NodeKind};
use crate::util::calculate_pointer_depth;
use crate::{tokenizer, util};

pub fn gen_lval(node: Node) {
    // check if node is an lvalue
    // address of the variable is pushed to the stack
    if let NodeKind::NdLvar = node.kind {
        println!("  mov rax, rbp");
        println!("  sub rax, {}", node.offset);
        println!("  push rax");
        return;
    }
    if let NodeKind::NdVardef = node.kind {
        println!("  mov rax, rbp");
        println!("  sub rax, {}", node.offset);
        println!("  push rax");
        return;
    }
    // if node is a dereference, push the address of the variable to the stack
    if let NodeKind::NdDeref = node.kind {
        gen(*node.rhs.unwrap());
        return;
    }
    util::error("not an lvalue");
}

pub fn gen(node: Node) {
    match node.kind {
        NodeKind::NdNum => {
            println!("  push {}", node.val);
            return;
        }
        NodeKind::NdLvar => {
            gen_lval(node.clone());

            println!("  pop rax");
            println!("  mov rax, [rax]");
            println!("  push rax");
            return;
        }
        NodeKind::NdVardef => {
            gen_lval(node.clone());

            println!("  pop rax");
            println!("  mov rax, [rax]");
            println!("  push rax");
            return;
        }
        NodeKind::NdAssign => {
            gen_lval(*node.clone().lhs.unwrap());
            gen(*node.clone().rhs.unwrap());

            println!("  pop rdi");
            println!("  pop rax");
            println!("  mov [rax], rdi");
            println!("  push rdi");
            return;
        }
        NodeKind::NdDeref => {
            gen(*node.clone().rhs.unwrap());
            println!("  pop rax");
            println!("  mov rax, [rax]");
            println!("  push rax");
            return;
        }
        NodeKind::NdAddr => {
            gen_lval(*node.clone().rhs.unwrap());
            return;
        }
        NodeKind::NdReturn => {
            gen(*node.clone().lhs.unwrap());
            println!("  pop rax");
            println!("  mov rsp, rbp");
            println!("  pop rbp");
            println!("  ret");
            return;
        }
        NodeKind::NdIf => {
            let label = util::gen_label();
            gen(*node.clone().lhs.unwrap());
            println!("  pop rax");
            println!("  cmp rax, 0");
            if let NodeKind::NdElse = node.clone().rhs.unwrap().kind {
                println!("  je .Lelse{}", label);
                gen(*node.clone().rhs.unwrap().lhs.unwrap());
                println!("  jmp .Lend{}", label);
                println!(".Lelse{}:", label);
                gen(*node.clone().rhs.unwrap().rhs.unwrap());
                println!(".Lend{}:", label);
            } else {
                println!("  je .Lend{}", label);
                gen(*node.clone().rhs.unwrap());
                println!(".Lend{}:", label);
            }
            return;
        }
        NodeKind::NdWhile => {
            let label = util::gen_label();
            println!(".Lbegin{}:", label);
            gen(*node.clone().lhs.unwrap());
            println!("  pop rax");
            println!("  cmp rax, 0");
            println!("  je .Lend{}", label);
            gen(*node.clone().rhs.unwrap());
            println!("  jmp .Lbegin{}", label);
            println!(".Lend{}:", label);
            return;
        }
        NodeKind::NdFor => {
            let label = util::gen_label();
            if let Some(init) = node.clone().lhs {
                gen(*init);
            }
            println!(".Lbegin{}:", label);
            if let Some(cond) = node.clone().rhs.unwrap().lhs {
                gen(*cond);
                println!("  pop rax");
                println!("  cmp rax, 0");
                println!("  je .Lend{}", label);
            }
            gen(*node.clone().rhs.unwrap().rhs.unwrap().rhs.unwrap());
            if let Some(inc) = node.clone().rhs.unwrap().rhs.unwrap().lhs {
                gen(*inc);
            }
            println!("  jmp .Lbegin{}", label);
            println!(".Lend{}:", label);
            return;
        }
        NodeKind::NdBlock => {
            for stmt in node.stmts {
                gen(stmt.clone());
                if let NodeKind::NdReturn = stmt.kind {
                    println!("  pop rax");
                }
            }
            return;
        }
        NodeKind::NdFunc => {
            let regs = vec!["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
            for arg in node.clone().stmts {
                gen(arg);
            }
            let mut i = node.clone().stmts.len();
            for _ in node.clone().stmts {
                i -= 1;
                println!("  pop rax");
                println!("  mov {}, rax", regs[i]);
            }
            println!("  call {}", node.clone().name);
            println!("  push rax");
            return;
        }
        NodeKind::NdAdd => {
            let (ptr_depth, num_nd) = calculate_pointer_depth(node.clone().lhs);
            if ptr_depth != 0
                || num_nd
                    .as_ref()
                    .and_then(|nd| nd.var_type.as_ref())
                    .map_or(false, |ty| matches!(ty.ty, tokenizer::TypeKind::TyPtr))
            {
                gen_ptr_binary_op(ptr_depth, num_nd, node.clone(), "add");
                return;
            }
            gen_binary_op(node.clone(), "add");
        }
        NodeKind::NdSub => {
            let (ptr_depth, num_nd) = calculate_pointer_depth(node.clone().lhs);
            if ptr_depth != 0
                || num_nd
                    .as_ref()
                    .and_then(|nd| nd.var_type.as_ref())
                    .map_or(false, |ty| matches!(ty.ty, tokenizer::TypeKind::TyPtr))
            {
                gen_ptr_binary_op(ptr_depth, num_nd, node.clone(), "sub");
                return;
            }
            gen_binary_op(node.clone(), "sub");
        }
        NodeKind::NdNeg => {
            gen_binary_op(node.clone(), "sub");
        }
        NodeKind::NdMul => {
            gen_binary_op(node.clone(), "imul");
        }
        NodeKind::NdDiv => {
            gen_binary_op(node.clone(), "idiv");
        }
        NodeKind::NdEq => {
            gen_cmp(node.clone(), "eq");
        }
        NodeKind::NdNe => {
            gen_cmp(node.clone(), "ne");
        }
        NodeKind::NdLt => {
            gen_cmp(node.clone(), "lt");
        }
        NodeKind::NdLe => {
            gen_cmp(node.clone(), "le");
        }
        NodeKind::NdGt => {
            gen_cmp(node.clone(), "gt");
        }
        NodeKind::NdGe => {
            gen_cmp(node.clone(), "ge");
        }
        _ => {}
    }

    println!("  push rax");
}

fn gen_ptr_binary_op(ptr_depth: i32, num_nd: Option<Box<Node>>, node: Node, op: &str) {
    if let Some(lhs) = node.lhs.clone() {
        gen(*lhs);
    }
    if let Some(rhs) = node.rhs.clone() {
        gen(*rhs);
    }
    println!("  pop rdi");

    if let Some(nd) = num_nd {
        let mut ty = nd.var_type.as_ref().unwrap();
        if ptr_depth > 0 {
            for _ in 0..ptr_depth + 1 {
                ty = ty.ptr_to.as_ref().unwrap();
            }
            println!("  imul rdi, {}", ty.size);
        } else if ptr_depth < 0 {
            if ptr_depth == -1 {
                println!("  imul rdi, {}", ty.size);
            } else {
                println!("  imul rdi, {}", 8);
            }
        } else {
            if let tokenizer::TypeKind::TyPtr = ty.ty {
                println!("  imul rdi, {}", ty.ptr_to.as_ref().unwrap().size);
            }
        }
        println!("  pop rax");
        println!("  {} rax, rdi", op);

        println!("  push rax");
    }
}

fn gen_binary_op(node: Node, op: &str) {
    if let Some(lhs) = node.lhs.clone() {
        gen(*lhs);
    }
    if let Some(rhs) = node.rhs.clone() {
        gen(*rhs);
    }
    println!("  pop rdi");
    println!("  pop rax");
    if op == "idiv" {
        println!("  cqo");
        println!("  idiv rdi");
    } else {
        println!("  {} rax, rdi", op);
    }
}

fn gen_cmp(node: Node, op: &str) {
    if let Some(lhs) = node.lhs.clone() {
        gen(*lhs);
    }
    if let Some(rhs) = node.rhs.clone() {
        gen(*rhs);
    }
    let op2 = match op {
        "eq" => "e",
        "ne" => "ne",
        "lt" => "l",
        "le" => "le",
        "gt" => "l",
        "ge" => "le",
        _ => "",
    };
    println!("  pop rdi");
    println!("  pop rax");
    if op == "gt" || op == "ge" {
        println!("  cmp rdi, rax");
    } else {
        println!("  cmp rax, rdi");
    }
    println!("  set{} al", op2);
    println!("  movzb rax, al");
}
