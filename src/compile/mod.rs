//! Compiles an AST to bytecode.

use crate::parse::{Expr, Ident};
use indexmap::IndexSet;
use num_bigint::BigInt;
use num_traits::identities::Zero;

/// Bytecode operations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    /// Push a predefined value to the stack.
    GetConstant(usize),
    /// Retrieve the value of a variable from the nearest scope it's defined, and push it to the stack. If the variable has not been defined, a [`VariableNotFound`](crate::interpret::ScriptError::VariableNotFound) error is thrown.
    GetIdent(usize),
    /// Pop a value from the stack and discard it.
    Drop,
    /// Duplicate the value at the top of the stack.
    Dup,
    /// Pop a value from the stack and assign it to a variable from the nearest scope. If the variable has not been defined, a [`VariableNotFound`](crate::interpret::ScriptError::VariableNotFound) error is thrown.
    Assign(usize),
    /// Pop a value from the stack and declare a variable in the current scope initialized with said value. If the variable has already been declared in the current scope, a [`VariableRedeclared`](crate::interpret::ScriptError::VariableRedeclared) error is thrown.
    Declare(usize),
    /// Pop a bytecode object from the stack and execute it. Additionally, some number of values are popped from the parent stack and pushed onto the child stack. This code may leave a single value on the stack as its return value.
    Call(usize),
}

#[derive(Debug, Clone, Copy)]
pub enum Intrinsic {
    Print,
    While,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    List,
    Last,
    Push,
    Len,
    Map,
    Fold,
    Filter,
    Zip,
    At,
}

pub const INTRINSIC_IDENTS: [(&str, Intrinsic); 16] = [
    ("print", Intrinsic::Print),
    ("while", Intrinsic::While),
    ("add", Intrinsic::Add),
    ("sub", Intrinsic::Sub),
    ("mul", Intrinsic::Mul),
    ("div", Intrinsic::Div),
    ("mod", Intrinsic::Mod),
    ("list", Intrinsic::List),
    ("last", Intrinsic::Last),
    ("push", Intrinsic::Push),
    ("len", Intrinsic::Len),
    ("map", Intrinsic::Map),
    ("fold", Intrinsic::Fold),
    ("filter", Intrinsic::Filter),
    ("zip", Intrinsic::Zip),
    ("at", Intrinsic::At),
];

impl Intrinsic {
    pub fn num_params(self) -> usize {
        match self {
            Self::List => 0,
            Self::Len | Self::Last | Self::Print => 1,
            Self::Add
            | Self::At
            | Self::Div
            | Self::Filter
            | Self::Fold
            | Self::Map
            | Self::Mod
            | Self::Mul
            | Self::Push
            | Self::Sub
            | Self::While
            | Self::Zip => 2,
        }
    }
}

/// A value which can be created and manipulated by user code.
#[derive(Debug, Clone)]
pub enum Value {
    /// A null value that is returned when there is no other possible value. The canonical representation of this value is the empty block `{}`.
    None,
    /// An integer value.
    Number(BigInt),
    /// A list of values.
    List(Vec<Value>),
    /// An executable bytecode value, as well as the number of arguments it requires (if any).
    Bytecode(Code, usize),
    /// An intrinsic function whose behavior is handled by the compiler/interpreter.
    Builtin(Intrinsic),
}

impl Value {
    pub fn truthiness(&self) -> bool {
        match self {
            Self::None => false,
            Self::Number(n) => !n.is_zero(),
            Self::List(list) => list.len() > 0,
            Self::Bytecode(..) | Self::Builtin(_) => true,
        }
    }
}

/// Represents an executable bytecode object, consisting of a list of bytecode operations and a collection of associated values.
#[derive(Debug, Default, Clone)]
pub struct Code {
    pub ops: Vec<Op>,
    //pub idents: Vec<Ident>,
    pub constants: Vec<Value>,
}

/// A boolean flag that signals whether the return value for an expression should be generated.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Return {
    /// Don't generate instructions to push a return a value.
    Discard,
    /// Generate instructions to push a return a value.
    Keep,
}

// impl Return {
//     /// Returns a boolean value representing whether to generate a return value.
//     pub fn keep(self) -> bool {
//         self == Self::Keep
//     }
// }

impl Code {
    fn add_expr(&mut self, expr: Expr, ident_list: &mut IndexSet<Ident>, return_mode: Return) {
        let does_return = return_mode == Return::Keep;
        match expr {
            Expr::Number(val) => {
                if does_return {
                    self.constants.push(Value::Number(val));
                    let index = self.constants.len() - 1;
                    self.ops.push(Op::GetConstant(index));
                }
            }
            Expr::Identifier(name) => {
                if does_return {
                    let name_index = insert_index(ident_list, name);
                    self.ops.push(Op::GetIdent(name_index));
                }
            }
            Expr::Assignment(lhs, rhs) => {
                self.add_expr(*rhs, ident_list, Return::Keep);
                if does_return {
                    self.ops.push(Op::Dup);
                }
                let name_index = insert_index(ident_list, lhs);
                self.ops.push(Op::Assign(name_index));
            }
            Expr::Declaration(lhs, rhs) => {
                self.add_expr(*rhs, ident_list, Return::Keep);
                if does_return {
                    self.ops.push(Op::Dup);
                }
                let name_index = insert_index(ident_list, lhs);
                self.ops.push(Op::Declare(name_index));
            }
            Expr::Block(exprs) => {
                let code = Self::compile(exprs, ident_list, return_mode);
                self.constants.push(Value::Bytecode(code, 0));
                let index = self.constants.len() - 1;
                self.ops.push(Op::GetConstant(index));
                // A block has no arguments to read from the stack.
                self.ops.push(Op::Call(0));
            }
            Expr::Lambda(params, body) => {
                if does_return {
                    let mut code = Self::default();
                    let num_params = params.len();
                    // Arguments pushed off the stack will be reversed.
                    for param in params.into_iter().rev() {
                        let name_index = insert_index(ident_list, param);
                        code.ops.push(Op::Declare(name_index));
                    }
                    code.add_expr(*body, ident_list, Return::Keep);
                    self.constants.push(Value::Bytecode(code, num_params));
                    let index = self.constants.len() - 1;
                    self.ops.push(Op::GetConstant(index));
                }
            }
            Expr::Application(func, args) => {
                let num_args = args.len();
                for arg in args {
                    self.add_expr(arg, ident_list, Return::Keep);
                }
                self.add_expr(*func, ident_list, Return::Keep);
                self.ops.push(Op::Call(num_args));
                if !does_return {
                    self.ops.push(Op::Drop);
                }
            }
        }
    }

    fn compile(
        mut exprs: Vec<Expr>,
        ident_list: &mut IndexSet<Ident>,
        return_mode: Return,
    ) -> Self {
        let does_return = return_mode == Return::Keep;
        let mut code = Self::default();
        if let Some(last_expr) = exprs.pop() {
            for expr in exprs {
                code.add_expr(expr, ident_list, Return::Discard);
            }
            code.add_expr(last_expr, ident_list, return_mode);
        } else if does_return {
            code.constants = vec![Value::None];
            code.ops = vec![Op::GetConstant(0)];
        }
        code
    }
}

/// Compiles a series of [`Expr`]s into a [`Code`] object and a list of identifiers used.
pub fn compile(exprs: Vec<Expr>) -> (Code, IndexSet<Ident>) {
    let mut idents = IndexSet::new();
    let code = Code::compile(exprs, &mut idents, Return::Discard);
    (code, idents)
}

fn insert_index<T>(set: &mut IndexSet<T>, value: T) -> usize
where
    T: Eq + std::hash::Hash,
{
    set.insert_full(value).0
}
