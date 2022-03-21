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

/// Errors within the interpreter. If this is ever publicly returned, that is a serious bug.
#[derive(Debug, Clone)]
pub enum InternalError {
    StackUnderflow,
    //StackOverflow,
    CallStackUnderflow,
    ConstantNotFound,
    CodeNotFound,
}

/// Errors generated from within user code.
#[derive(Debug, Clone)]
pub enum ScriptError {
    VariableNotFound,
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
        // Access `ops` by index to prevent simultaneous mutable references.
        // This could also be done with mem::take to the same effect.
        loop {
            if let Some(op) = self.code.ops.get(self.op_pointer) {
                let op = op.clone();
                self.op_pointer += 1;
                println!("awa {:?}", self);
                println!("using op {:?}", op);
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
                    // No other error should be returned.
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
                        //let existing_value = entry.get();
                        // panic!(
                        //     "Attempted to declare variable {} with value {:?} but was already assigned with value of {:?}",
                        //     ident,
                        //     value,
                        //     existing_value,
                        // );
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
            //panic!("Could not find variable {}", name)
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
        let mut parent = mem::replace(self, child);
        //self.code = mem::take(&mut parent.code);
        self.stack = mem::take(&mut parent.stack);
        self.parent = Some((Box::new(parent), ptr));
        self.op_pointer = 0;
    }

    fn exit_subroutine(&mut self) -> InternalResult<()> {
        let (parent, ptr) = mem::take(&mut self.parent).ok_or(InternalError::CallStackUnderflow)?;
        let child = mem::replace(self, *parent);
        //self.code = child.code;
        self.stack = child.stack;
        self.op_pointer = ptr;
        Ok(())
    }
}
