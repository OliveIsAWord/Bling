//! Interprets and executes bytecode.

use crate::compile::{Code, Op, Value};
use crate::parse::Ident;
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
}

/// Errors within the interpreter. If this is ever publicly returned, that would constitute a serious bug.
#[derive(Debug, Clone)]
pub enum InternalError {
    /// An operation that pops a value off the stack was executed when the stack was empty.
    StackUnderflow,
    //StackOverflow,
    /// The interpreter expected to return from a subroutine when no caller existed.
    CallStackUnderflow,
    /// An operation requested a constant value that does not exist.
    ConstantNotFound,
    /// An operation requested a [Code] object that does not exist.
    CodeNotFound,
}

/// Errors generated from within user code.
#[derive(Debug, Clone)]
pub enum ScriptError {
    /// A variable that had not been defined was read or assigned to.
    VariableNotFound,
    /// A variable was declared twice in the same scope.
    VariableRedeclared,
}

pub type InternalResult<T> = Result<T, InternalError>;
pub type ScriptResult<T> = Result<T, ScriptError>;
pub type ExecResult<T> = InternalResult<ScriptResult<T>>;

impl Executor {
    pub fn from_code(code: Code) -> Self {
        Self {
            code,
            ..Default::default()
        }
    }

    pub fn run(&mut self) -> ExecResult<()> {
        loop {
            if let Some(op) = self.code.ops.get(self.op_pointer) {
                let op = op.clone();
                self.op_pointer += 1;
                println!("Current State:\n{:?}\n", self);
                println!("Running Op: {:?}", op);
                match self.run_step(op) {
                    Ok(Ok(())) => (),
                    e => return e,
                }
            } else {
                match self.exit_subroutine() {
                    // Successfully returned to caller.
                    Ok(()) => (),
                    // No caller to return to, so exit.
                    Err(InternalError::CallStackUnderflow) => return Ok(Ok(())),
                    // No other error should be returned by `exit_subroutine`.
                    Err(e) => return Err(e),
                }
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
            // Op::Dup => {
            //     let val = self.stack.last().unwrap().clone();
            //     self.stack.push(val);
            // }
            Op::Declare(ident) => {
                let value = self.stack.pop().ok_or(InternalError::StackUnderflow)?;
                match self.scope.entry(ident) {
                    Entry::Occupied(_entry) => {
                        return Ok(Err(ScriptError::VariableRedeclared));
                    }
                    Entry::Vacant(space) => {
                        space.insert(value);
                    }
                }
            }
            Op::Assign(ident) => {
                let value = self.stack.pop().ok_or(InternalError::StackUnderflow)?;
                match self.lookup_value_mut(ident) {
                    Ok(entry_ref) => {
                        *entry_ref = value;
                    }
                    Err(e) => {
                        return Ok(Err(e));
                    }
                }
            }
            Op::Call(code_index) => {
                let block_code = self
                    .code
                    .codes
                    .get(code_index)
                    .ok_or(InternalError::CodeNotFound)?
                    .clone();
                self.enter_subroutine(block_code);
            }
        }
        Ok(Ok(()))
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

    fn enter_subroutine(&mut self, routine: Code) {
        let ptr = self.op_pointer;
        let child = Self::from_code(routine);
        // `self` becomes `parent`, and `child` becomes `self`
        let mut parent = mem::replace(self, child);
        self.stack = mem::take(&mut parent.stack);
        self.parent = Some((Box::new(parent), ptr));
        self.op_pointer = 0;
    }

    fn exit_subroutine(&mut self) -> InternalResult<()> {
        let (parent, ptr) = mem::take(&mut self.parent).ok_or(InternalError::CallStackUnderflow)?;
        let child = mem::replace(self, *parent);
        self.stack = child.stack;
        self.op_pointer = ptr;
        Ok(())
    }
}
