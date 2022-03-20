//! Compiling AST to bytecode.

use crate::parse::{Expr, Ident};

/// Bytecode operations.
#[derive(Debug)]
pub enum Op {
    GetConstant(usize),
    //Assign(usize),
    Declare(usize),
    //Call,
}

#[derive(Debug, Clone)]
pub enum Value {
    // None,
    Number(i64),
}

/// Executable bytecode environment.
#[derive(Debug, Default)]
pub struct Code {
    pub ops: Vec<Op>,
    pub idents: Vec<Ident>,
    pub constants: Vec<Value>,
}

/// Compiles a single [Expr] into a [Code] object.
pub fn compile_expr(expr: Expr) -> Code {
    match expr {
        Expr::Declaration(lhs, rhs) => {
            let val = if let Expr::Number(x) = *rhs {
                x
            } else {
                panic!();
            };
            let ops = vec![Op::GetConstant(0), Op::Declare(0)];
            let idents = vec![lhs];
            let constants = vec![Value::Number(val)];
            Code {
                ops,
                idents,
                constants,
            }
        }
        _ => todo!(),
    }
}
