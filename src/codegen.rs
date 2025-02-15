use crate::parser::{Node, NodeKind};
use crate::tokenizer;

pub fn gen_lval(node: Node) {
    // check if node is an lvalue
    // address of the variable is pushed to the stack
    if let NodeKind::NdLvar = node.kind {
        println!("  mov rax, rbp");
        println!("  sub rax, {}", node.offset);
        println!("  push rax");
        return;
    }
    tokenizer::error("not an lvalue");
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
            let label = tokenizer::gen_label();
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
            let label = tokenizer::gen_label();
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
            let label = tokenizer::gen_label();
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
        _ => {}
    }

    if let Some(lhs) = node.lhs {
        gen(*lhs);
    }
    if let Some(rhs) = node.rhs {
        gen(*rhs);
    }

    println!("  pop rdi");
    println!("  pop rax");

    match node.kind {
        NodeKind::NdAdd => {
            println!("  add rax, rdi");
        }
        NodeKind::NdSub | NodeKind::NdNeg => {
            println!("  sub rax, rdi");
        }
        NodeKind::NdMul => {
            println!("  imul rax, rdi");
        }
        NodeKind::NdDiv => {
            println!("  cqo");
            println!("  idiv rdi");
        }
        NodeKind::NdEq => {
            println!("  cmp rax, rdi");
            println!("  sete al");
            println!("  movzb rax, al");
        }
        NodeKind::NdNe => {
            println!("  cmp rax, rdi");
            println!("  setne al");
            println!("  movzb rax, al");
        }
        NodeKind::NdLt => {
            println!("  cmp rax, rdi");
            println!("  setl al");
            println!("  movzb rax, al");
        }
        NodeKind::NdLe => {
            println!("  cmp rax, rdi");
            println!("  setle al");
            println!("  movzb rax, al");
        }
        NodeKind::NdGt => {
            println!("  cmp rdi, rax");
            println!("  setl al");
            println!("  movzb rax, al");
        }
        NodeKind::NdGe => {
            println!("  cmp rdi, rax");
            println!("  setle al");
            println!("  movzb rax, al");
        }
        _ => {}
    }

    println!("  push rax");
}
