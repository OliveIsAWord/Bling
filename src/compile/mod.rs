//! Compiling AST to bytecode.

use crate::parse::{Expr, Ident};

/// Bytecode operations.
#[derive(Debug)]
pub enum Op {
    GetConstant(usize),
    GetIdent(usize),
    Assign(usize),
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

impl Code {
    fn add_expr(&mut self, expr: Expr, returns_value: bool) {
        match expr {
            Expr::Number(val) => if returns_value {
                self.constants.push(Value::Number(val));
                let index = self.constants.len() - 1;
                self.ops.push(Op::GetConstant(index));
            }
            Expr::Identifier(name) => if returns_value {
                let index = self.add_ident(name);
                self.ops.push(Op::GetIdent(index));
            }
            Expr::Assignment(lhs, rhs) => {
                self.add_expr(*rhs, true);
                let index = self.add_ident(lhs);
                self.ops.push(Op::Assign(index));
                if returns_value {
                    self.ops.push(Op::GetIdent(index));
                }
            }
            Expr::Declaration(lhs, rhs) => {
                self.add_expr(*rhs, true);
                let index = self.add_ident(lhs);
                self.ops.push(Op::Declare(index));
                if returns_value {
                    self.ops.push(Op::GetIdent(index));
                }
            }
            // Expr::Block(exprs) => {
            //     let (last_expr, body) = exprs.split_last();
            // }
            _ => todo!(),
        }
    }
    fn add_ident(&mut self, name: Ident) -> usize {
        self.idents
            .iter()
            .position(|x| x == &name)
            .unwrap_or_else(|| {
                self.idents.push(name);
                self.idents.len() - 1
            })
    }
}

/// Compiles a series of [Expr]s into a [Code] object.
pub fn compile(exprs: Vec<Expr>) -> Code {
    let mut code = Code::default();
    for expr in exprs {
        code.add_expr(expr, false);
    }
    code
}
