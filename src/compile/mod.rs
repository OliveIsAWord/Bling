//! Compiles an AST to bytecode.

use crate::parse::{Expr, Ident};

/// Bytecode operations.
#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    /// Push a predefined value to the stack.
    GetConstant(usize),
    /// Retrieve the value of a variable from the nearest scope it's defined, and push it to the stack. If the variable has not been defined, a [`VariableNotFound`](crate::interpret::ScriptError::VariableNotFound) error is thrown.
    GetIdent(Ident),
    /// Pop a value from the stack and assign it to a variable from the nearest scope. If the variable has not been defined, a [`VariableNotFound`](crate::interpret::ScriptError::VariableNotFound) error is thrown.
    Assign(Ident),
    /// Pop a value from the stack and declare a variable in the current scope initialized with said value. If the variable has already been declared in the current scope, a [`VariableRedeclared`](crate::interpret::ScriptError::VariableRedeclared) error is thrown.
    Declare(Ident),
    /// Retrieve a predefined bytecode object and execute it. This bytecode can leave an extra value on the stack as its return value.
    Call(usize),
}

/// A value which can be created and manipulated by user code.
#[derive(Debug, Clone)]
pub enum Value {
    /// A null value that is returned when there is no other possible value. The canonical representation of this value is the empty block `{}`.
    None,
    /// An integer. Note that in a future version, this value will be upgraded to a bigint.
    Number(i64),
}

/// Represents an executable bytecode object, consisting of a list of bytecode operations and a collection of associated values.
#[derive(Debug, Default, Clone)]
pub struct Code {
    pub ops: Vec<Op>,
    pub idents: Vec<Ident>,
    pub constants: Vec<Value>,
    pub codes: Vec<Code>, // FIXME: Better name
}

/// A boolean flag that signals whether the return value for an expression should be generated.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Return {
    /// Don't generate instructions to push a return a value.
    Discard,
    /// Generate instructions to push a return a value.
    Keep,
}

impl Return {
    /// Returns a boolean value representing whether to generate a return value.
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
                    self.ops.push(Op::GetIdent(name));
                }
            }
            Expr::Assignment(lhs, rhs) => {
                self.add_expr(*rhs, Return::Keep);
                self.ops.push(Op::Assign(lhs.clone()));
                if returns.keep() {
                    self.ops.push(Op::GetIdent(lhs));
                }
            }
            Expr::Declaration(lhs, rhs) => {
                self.add_expr(*rhs, Return::Keep);
                self.ops.push(Op::Declare(lhs.clone()));
                if returns.keep() {
                    self.ops.push(Op::GetIdent(lhs));
                }
            }
            Expr::Block(exprs) => {
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
}

/// Compiles a series of [`Expr`]s into a [`Code`] object.
pub fn compile(exprs: Vec<Expr>) -> Code {
    Code::compile(exprs, Return::Discard)
}
