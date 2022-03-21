//! Compiling AST to bytecode.

use crate::parse::{Expr, Ident};

/// Bytecode operations.
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    GetConstant(usize),
    GetIdent(Ident),
    Assign(Ident),
    Declare(Ident),
    Call(usize),
}

#[derive(Debug, Clone)]
pub enum Value {
    None,
    Number(i64),
}

/// Executable bytecode environment.
#[derive(Debug, Default, Clone)]
pub struct Code {
    pub ops: Vec<Op>,
    pub idents: Vec<Ident>,
    pub constants: Vec<Value>,
    pub codes: Vec<Code>, // FIXME: Better name
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Return {
    Discard,
    Keep,
}

impl Return {
    pub fn keep(self) -> bool {
        self == Self::Keep
    }
}

impl Code {
    fn add_expr(&mut self, expr: Expr, returns: Return) {
        match expr {
            Expr::Number(val) => {
                if returns.keep() {
                    self.constants.push(Value::Number(val));
                    let index = self.constants.len() - 1;
                    self.ops.push(Op::GetConstant(index));
                }
            }
            Expr::Identifier(name) => {
                if returns.keep() {
                    //let index = self.add_ident(name);
                    //self.ops.push(Op::GetIdent(index));
                    self.ops.push(Op::GetIdent(name));
                }
            }
            Expr::Assignment(lhs, rhs) => {
                // self.add_expr(*rhs, Return::Keep);
                // let index = self.add_ident(lhs);
                // self.ops.push(Op::Assign(index));
                // if returns.keep() {
                //     self.ops.push(Op::GetIdent(index));
                // }
                self.add_expr(*rhs, Return::Keep);
                self.ops.push(Op::Assign(lhs.clone()));
                if returns.keep() {
                    self.ops.push(Op::GetIdent(lhs));
                }
            }
            Expr::Declaration(lhs, rhs) => {
                // self.add_expr(*rhs, Return::Keep);
                // let index = self.add_ident(lhs);
                // self.ops.push(Op::Declare(index));
                // if returns.keep() {
                //     self.ops.push(Op::GetIdent(index));
                // }
                self.add_expr(*rhs, Return::Keep);
                self.ops.push(Op::Declare(lhs.clone()));
                if returns.keep() {
                    self.ops.push(Op::GetIdent(lhs));
                }
            }
            Expr::Block(exprs) => {
                // if let Some(last_expr) = exprs.pop() {
                //     for body_expr in exprs {
                //         self.add_expr(body_expr, Return::Discard);
                //     }
                //     self.add_expr(last_expr, returns_value);
                // } else if returns.keep() {
                //     self.constants.push(Value::None);
                //     let index = self.constants.len() - 1;
                //     self.ops.push(Op::GetConstant(index));
                // }
                let code = Self::compile(exprs, returns);
                self.codes.push(code);
                let index = self.codes.len() - 1;
                self.ops.push(Op::Call(index));
            }
            _ => todo!(),
        }
    }

    fn compile(mut exprs: Vec<Expr>, returns: Return) -> Self {
        let mut code = Self::default();
        if let Some(last_expr) = exprs.pop() {
            for expr in exprs {
                code.add_expr(expr, Return::Discard);
            }
            code.add_expr(last_expr, returns);
        } else if returns.keep() {
            code.constants = vec![Value::None];
            code.ops = vec![Op::GetConstant(0)];
        }
        code
    }

    // fn add_ident(&mut self, name: Ident) -> usize {
    //     self.idents
    //         .iter()
    //         .position(|x| x == &name)
    //         .unwrap_or_else(|| {
    //             self.idents.push(name);
    //             self.idents.len() - 1
    //         })
    // }
}

/// Compiles a series of [Expr]s into a [Code] object.
pub fn compile(exprs: Vec<Expr>) -> Code {
    Code::compile(exprs, Return::Discard)
}
