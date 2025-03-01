use crate::ast::{Node, NodeKind};
use crate::sema::TypeKind;
use crate::util;

fn load(node: Node) {
    println!("  pop rax");
    if let TypeKind::TyArray = node.clone().var_type.as_ref().unwrap().ty {
    } else {
        println!("  mov rax, [rax]");
    }
    println!("  push rax");
}

fn store(node: Node) {
    println!("  pop rdi");
    println!("  pop rax");

    println!("  mov [rax], rdi");
    println!("  push rdi");
}

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

            load(node);
            return;
        }
        NodeKind::NdVardef => {
            gen_lval(node.clone());

            load(node);
            return;
        }
        NodeKind::NdAssign => {
            gen_lval(*node.clone().lhs.unwrap());
            gen(*node.clone().rhs.unwrap());

            store(node.clone());
            return;
        }
        NodeKind::NdDeref => {
            gen(*node.clone().rhs.unwrap());

            load(node);
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
            let lty = node.clone().lhs.unwrap().var_type.unwrap();
            let rty = node.clone().rhs.unwrap().var_type.unwrap();
            match (lty.ty, rty.ty) {
                (TypeKind::TyInt, TypeKind::TyInt) => {
                    gen_binary_op(node.clone(), "add");
                }
                _ => {
                    gen_ptr_binary_op(node.clone(), "add");
                }
            }
        }
        NodeKind::NdSub => {
            let lty = node.clone().lhs.unwrap().var_type.unwrap();
            let rty = node.clone().rhs.unwrap().var_type.unwrap();
            match (lty.ty, rty.ty) {
                (TypeKind::TyInt, TypeKind::TyInt) => {
                    gen_binary_op(node.clone(), "sub");
                }
                _ => {
                    gen_ptr_binary_op(node.clone(), "sub");
                }
            }
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

fn gen_ptr_binary_op(node: Node, op: &str) {
    if let Some(lhs) = node.lhs.clone() {
        gen(*lhs);
    }
    if let Some(rhs) = node.rhs.clone() {
        gen(*rhs);
    }
    println!("  pop rdi");

    let ty = node.lhs.unwrap().var_type.unwrap().ptr_to.unwrap();
    let ty_size = ty.size;
    println!("  imul rdi, {}", ty_size);
    println!("  pop rax");
    println!("  {} rax, rdi", op);
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
