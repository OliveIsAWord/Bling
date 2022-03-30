//! Interprets and executes bytecode.

use crate::compile::{Code, Intrinsic, Op, Value, INTRINSIC_IDENTS};
use crate::parse::Ident;
use num_bigint::BigInt;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem;

#[derive(Debug, Default)]
pub struct Executor {
    code: Code,
    op_pointer: usize,
    scope: HashMap<Ident, Value>,
    stack: Vec<Value>,
    parent: Option<(Box<Executor>, usize)>,
    depth: usize,
}

/// Errors within the interpreter. If this is ever publicly returned, that would constitute a serious bug.
#[derive(Debug, Clone)]
pub enum InternalError {
    /// An operation that pops a value off the stack was executed when the stack was empty.
    StackUnderflow,
    /// The interpreter expected to return from a subroutine when no caller existed.
    CallStackUnderflow,
    /// An operation requested a constant value that does not exist.
    ConstantNotFound,
    // /// Execution halted while values were still on the stack.
    // StackLeftovers,
}

/// Errors generated from within user code.
#[derive(Debug, Clone)]
pub enum ScriptError {
    /// A variable that had not been defined was read or assigned to.
    VariableNotFound,
    /// A variable was declared twice in the same scope.
    VariableRedeclared,
    /// The code attempted to call a non-code/non-builtin value.
    TypeNotCallable,
    /// The code attempted to call a function with the wrong number of arguments.
    ArgumentCount,
    /// One or more arguments had an invalid type for the function called.
    ArgumentType,
}

pub type InternalResult<T> = Result<T, InternalError>;
pub type ScriptResult<T> = Result<T, ScriptError>;
pub type ExecResult<T> = InternalResult<ScriptResult<T>>;

macro_rules! double_try {
    ($e:expr) => {
        match $e {
            Ok(Ok(v)) => v,
            Ok(Err(e)) => return Ok(Err(e)),
            Err(e) => return Err(e),
        }
    };
}

macro_rules! arithmetic_intrinsic {
    ($self:ident, $oper:expr) => {{
        let val2 = $self.pop_stack()?;
        let val1 = $self.pop_stack()?;
        match (val1, val2) {
            (Number(x), Number(y)) => $oper(x, y),
            _ => return Ok(Err(ScriptError::ArgumentType)),
        }
    }};
}

impl Executor {
    pub fn from_code(code: Code) -> Self {
        Self {
            code,
            ..Self::default()
        }
    }

    pub fn initialize_builtins(&mut self) {
        for (name, intrinsic) in INTRINSIC_IDENTS {
            self.scope.insert(name.into(), Value::Builtin(intrinsic));
        }
    }

    pub fn run(&mut self) -> ExecResult<()> {
        loop {
            if let Some(op) = self.code.ops.get(self.op_pointer) {
                let op = op.clone();
                self.op_pointer += 1;
                //println!("Current State:\n{:?}\n", self);
                //println!("Running Op: {:?}", op);
                double_try!(self.run_step(op));
            } else if self.depth > 0 {
                self.exit_subroutine()?;
            } else {
                return Ok(Ok(()));
            }
        }
    }

    fn run_step(&mut self, op: Op) -> ExecResult<()> {
        match op {
            Op::GetConstant(val_index) => {
                let val = self
                    .code
                    .constants
                    .get(val_index)
                    .ok_or(InternalError::ConstantNotFound)?;
                self.stack.push(val.clone());
            }
            Op::GetIdent(ident) => match self.lookup_value(ident) {
                Ok(val) => {
                    let val = val.clone();
                    self.stack.push(val);
                }
                Err(e) => {
                    return Ok(Err(e));
                }
            },
            Op::Drop => {
                self.pop_stack()?;
            }
            // Op::Dup => {
            //     let val = self.pop_stack()?;
            //     self.stack.push(val);
            // }
            Op::Declare(ident) => {
                let value = self.pop_stack()?;
                match self.scope.entry(ident) {
                    // If the variable is already defined *in this scope*, it's a redeclaration.
                    Entry::Occupied(_entry) => {
                        return Ok(Err(ScriptError::VariableRedeclared));
                    }
                    // Otherwise, initialize the variable with the given value.
                    Entry::Vacant(space) => {
                        space.insert(value);
                    }
                }
            }
            Op::Assign(ident) => {
                let value = self.pop_stack()?;
                match self.lookup_value_mut(ident) {
                    // If the variable is already defined, then reassign it.
                    Ok(entry_ref) => {
                        *entry_ref = value;
                    }
                    Err(e) => {
                        return Ok(Err(e));
                    }
                }
            }
            Op::Call(num_args) => match self.pop_stack()? {
                Value::Bytecode(code, num_params) => {
                    if num_params != num_args {
                        return Ok(Err(ScriptError::ArgumentCount));
                    }
                    self.enter_subroutine(code, num_args);
                }
                Value::Builtin(intrinsic) => {
                    if intrinsic.num_params() != num_args {
                        return Ok(Err(ScriptError::ArgumentCount));
                    }
                    double_try!(self.run_builtin(intrinsic));
                }
                _ => return Ok(Err(ScriptError::TypeNotCallable)),
            },
        }
        Ok(Ok(()))
    }

    fn pop_stack(&mut self) -> InternalResult<Value> {
        self.stack.pop().ok_or(InternalError::StackUnderflow)
    }

    fn lookup_value(&self, name: Ident) -> ScriptResult<&Value> {
        if let Some(val) = self.scope.get(&name) {
            Ok(val)
        } else if let Some(parent) = &self.parent {
            parent.0.lookup_value(name)
        } else {
            Err(ScriptError::VariableNotFound)
        }
    }

    fn lookup_value_mut(&mut self, name: Ident) -> ScriptResult<&mut Value> {
        if let Some(val) = self.scope.get_mut(&name) {
            Ok(val)
        } else if let Some(parent) = &mut self.parent {
            parent.0.lookup_value_mut(name)
        } else {
            Err(ScriptError::VariableNotFound)
        }
    }

    fn enter_subroutine(&mut self, routine: Code, _num_args: usize) {
        let ptr = self.op_pointer;
        let child = Self::from_code(routine);
        // `self` becomes `parent`, and `child` becomes `self`
        let mut parent = mem::replace(self, child);
        self.stack = mem::take(&mut parent.stack);
        self.parent = Some((Box::new(parent), ptr));
        self.op_pointer = 0;
        self.depth += 1;
    }

    fn exit_subroutine(&mut self) -> InternalResult<()> {
        let (parent, ptr) = mem::take(&mut self.parent).ok_or(InternalError::CallStackUnderflow)?;
        let child = mem::replace(self, *parent);
        self.stack = child.stack;
        self.op_pointer = ptr;
        // self.depth -= 1;
        Ok(())
    }

    fn run_code_object(&mut self, code: Code) -> ExecResult<Value> {
        // Run as if we are the main execution.
        let depth = self.depth;
        self.enter_subroutine(code, 0);
        self.depth = 0;
        double_try!(self.run());
        self.exit_subroutine()?;
        self.depth = depth;
        self.pop_stack().map(Ok)
        //Ok(Ok(()))
    }

    fn run_builtin(&mut self, intrinsic: Intrinsic) -> ExecResult<()> {
        use Value::{Bytecode, Number};
        let return_value = match intrinsic {
            Intrinsic::Print => {
                let val = self.pop_stack()?;
                println!("{:?}", val);
                Value::None
            }
            Intrinsic::Add => arithmetic_intrinsic!(self, |x, y| Number(x + y)),
            Intrinsic::Sub => arithmetic_intrinsic!(self, |x, y| Number(x - y)),
            Intrinsic::Mul => arithmetic_intrinsic!(self, |x, y| Number(x * y)),
            Intrinsic::Div => arithmetic_intrinsic!(self, |x: BigInt, y: BigInt| x
                .checked_div(&y)
                .map_or(Value::None, Number)),
            Intrinsic::While => {
                let val2 = self.pop_stack()?;
                let val1 = self.pop_stack()?;
                match (val1, val2) {
                    (Bytecode(condition, 0), Bytecode(body, 0)) => {
                        let mut output = Value::None;
                        while double_try!(self.run_code_object(condition.clone())).truthiness() {
                            output = double_try!(self.run_code_object(body.clone()));
                        }
                        output
                    }
                    _ => return Ok(Err(ScriptError::ArgumentType)),
                }
            }
        };
        self.stack.push(return_value);
        Ok(Ok(()))
    }
}
