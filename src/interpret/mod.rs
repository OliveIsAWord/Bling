//! Interprets and executes bytecode.

mod intrinsics;
#[macro_use]
mod macros;

use crate::compile::{Code, Intrinsic, Op, Value, INTRINSIC_IDENTS};
use indexmap::IndexSet;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::mem;

#[derive(Debug, Default, Clone)]
pub struct Executor {
    code: Code,
    idents: IndexSet<String>,
    op_pointer: usize,
    scope: HashMap<usize, Value>,
    stack: Vec<Value>,
    parent: Option<(Box<Self>, usize)>,
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
    /// One or more arguments had the right type but an invalid value for the function called.
    ArgumentValue,
}

pub type InternalResult<T> = Result<T, InternalError>;
pub type ScriptResult<T> = Result<T, ScriptError>;
pub type ExecResult<T> = InternalResult<ScriptResult<T>>;

impl Executor {
    pub fn from_code(code: Code, idents: IndexSet<String>) -> Self {
        Self {
            code,
            idents,
            ..Self::default()
        }
    }

    /// Adds every builtin function whose names appear anywhere in the current code to the current scope.
    pub fn initialize_builtins(&mut self) {
        for (name, intrinsic) in INTRINSIC_IDENTS {
            if let Some(name_index) = self.idents.get_index_of(name) {
                self.scope.insert(name_index, Value::Builtin(intrinsic));
            }
        }
    }

    pub fn run(&mut self) -> ExecResult<()> { // 58.15%
        loop {
            if let Some(&op) = self.code.ops.get(self.op_pointer) {
                self.op_pointer += 1;
                //println!("Current State:\n{:?}\n", self);
                //println!("Running Op: {:?}", op);
                double_try!(self.run_step(op)); // 55.02%
            } else if self.depth > 0 {
                self.exit_subroutine()?;
            } else {
                return Ok(Ok(()));
            }
        }
    }

    fn run_step(&mut self, op: Op) -> ExecResult<()> { // 55.02%
        // run_builtin 19.05%
        // Lookup value 13.16%
        // Value clone 7.99%
        // Lookup value mut 1.09%
        // Push element to vec 1.03%
        // Pop emelemt from vec 0.91%
        // Executor::run 0.42%
        // Value drop_in_place 0.36%
        // slice::get 0.04%
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
                    eprintln!(
                        "{:?} accessed but not defined",
                        self.idents.get_index(ident)
                    );
                    return Ok(Err(e));
                }
            },
            Op::Drop => {
                self.pop_stack()?;
            }
            Op::Dup => {
                let val = self.peek_stack()?.clone();
                self.stack.push(val);
            }
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
                        eprintln!(
                            "{:?} assigned to but not defined",
                            self.idents.get_index(ident)
                        );
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

    fn peek_stack(&self) -> InternalResult<&Value> {
        self.stack.last().ok_or(InternalError::StackUnderflow)
    }

    fn lookup_value(&self, name_index: usize) -> ScriptResult<&Value> {
        self.scope
            .get(&name_index)
            .or_else(|| {
                self.parent
                    .as_ref()
                    .and_then(|p| p.0.lookup_value(name_index).ok())
            })
            .ok_or(ScriptError::VariableNotFound)
    }

    fn lookup_value_mut(&mut self, name_index: usize) -> ScriptResult<&mut Value> {
        self.scope
            .get_mut(&name_index)
            .or_else(|| {
                self.parent
                    .as_mut()
                    .and_then(|p| p.0.lookup_value_mut(name_index).ok())
            })
            .ok_or(ScriptError::VariableNotFound)
    }

    // fn lookup_value_mut(&mut self, name_index: usize) -> ScriptResult<&mut Value> {
    //     if let Some(val) = self.scope.get_mut(&name_index) {
    //         Ok(val)
    //     } else if let Some(parent) = &mut self.parent {
    //         parent.0.lookup_value_mut(name_index)
    //     } else {
    //         Err(ScriptError::VariableNotFound)
    //     }
    // }

    fn enter_subroutine(&mut self, routine: Code, _num_args: usize) { // 19.32%
        let ptr = self.op_pointer;
        let idents = mem::take(&mut self.idents); // mem::take 1.25%
        let child = Self::from_code(routine, idents); // 8.11%
        // `self` becomes `parent`, and `child` becomes `self`
        let mut parent = mem::replace(self, child); // 1.60%
        self.stack = mem::take(&mut parent.stack);
        self.parent = Some((Box::new(parent), ptr)); // 2.19%
        self.op_pointer = 0;
        self.depth += 1;
    }

    fn exit_subroutine(&mut self) -> InternalResult<()> { // 13.24%
        let (parent, ptr) = mem::take(&mut self.parent).ok_or(InternalError::CallStackUnderflow)?; // mem::take 0.08%, ok_or 0.08%
        let child = mem::replace(self, *parent); // 2.61%
        self.stack = child.stack;
        self.op_pointer = ptr;
        // self.depth -= 1;
        Ok(())
        // freeing and dropping 8.99%
    }

    fn run_code_object(&mut self, code: Code) -> ExecResult<Value> { // 91.65%
        // Run as if we are the main execution.
        let depth = self.depth;
        self.enter_subroutine(code, 0); // 19.32%
        self.depth = 0;
        double_try!(self.run()); // 58.15%
        self.exit_subroutine()?; // 13.24%
        self.depth = depth;
        self.pop_stack().map(Ok) // 0.65%
    }

    fn run_builtin(&mut self, intrinsic: Intrinsic) -> ExecResult<()> { // 19.05%
        let return_value = double_try!(match intrinsic {
            Intrinsic::Print => intrinsics::print(self),
            Intrinsic::While => intrinsics::while_loop(self),
            Intrinsic::Add => intrinsics::add(self),
            Intrinsic::Sub => intrinsics::sub(self),
            Intrinsic::Mul => intrinsics::mul(self),
            Intrinsic::Div => intrinsics::div(self),
            Intrinsic::Mod => intrinsics::modulo(self),
            Intrinsic::List => intrinsics::list(self),
            Intrinsic::Last => intrinsics::last(self),
            Intrinsic::Push => intrinsics::push(self),
            Intrinsic::Len => intrinsics::len(self),
            Intrinsic::Map => intrinsics::map(self),
            Intrinsic::Fold => intrinsics::fold(self),
            Intrinsic::Filter => intrinsics::filter(self),
            Intrinsic::Zip => intrinsics::zip(self),
            Intrinsic::At => intrinsics::at(self),
        });
        self.stack.push(return_value);
        Ok(Ok(()))
    }
}
