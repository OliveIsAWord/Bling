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

enum InternalError {
    StackUnderflow,
    StackOverflow,
}
enum ScriptError {
    VariableNotFound,
    VariableRedeclared,
}

impl Executor {
    pub fn from_code(code: Code) -> Self {
        Self {
            code,
            ..Default::default()
        }
    }

    pub fn run(&mut self) {
        // Access `ops` by index to prevent simultaneous mutable references.
        // This could also be done with mem::take to the same effect.
        loop {
            if let Some(op) = self.code.ops.get(self.op_pointer) {
                let op = op.clone();
                self.op_pointer += 1;
                println!("awa {:?}", self);
                println!("using op {:?}", op);
                self.run_step(op);
            } else if self.parent.is_some() {
                self.exit_subroutine();
            } else {
                break;
            }
        }
    }

    fn run_step(&mut self, op: Op) {
        match op {
            Op::GetConstant(val_index) => {
                let val = self.code.constants.get(val_index).unwrap();
                self.stack.push(val.clone());
            }
            Op::GetIdent(ident) => {
                let val = self.lookup_value(ident).clone();
                self.stack.push(val);
            }
            // Op::Dup => {
            //     let val = self.stack.last().unwrap().clone();
            //     self.stack.push(val);
            // }
            Op::Declare(ident) => {
                let value = self.stack.pop().unwrap();
                match self.scope.entry(ident.clone()) {
                    Entry::Occupied(entry) => {
                        //let ident_name = self.code.idents.get(ident_index).unwrap();
                        let existing_value = entry.get();
                        panic!(
                            "Attempted to declare variable {} with value {:?} but was already assigned with value of {:?}",
                            ident,
                            value,
                            existing_value,
                        );
                    }
                    Entry::Vacant(space) => {
                        space.insert(value);
                    }
                }
            }
            Op::Assign(ident) => {
                let value = self.stack.pop().unwrap();
                *self.lookup_value_mut(ident) = value;
                // match self.scope.entry(ident.clone()) {
                //     Entry::Occupied(mut entry) => {
                //         entry.insert(value);
                //     }
                //     Entry::Vacant(_) => {
                //         //let ident_name = self.code.idents.get(ident_index).unwrap();
                //         panic!(
                //             "Attempted to assign variable {} before it was declared",
                //             ident,
                //         );
                //     }
                // }
            }
            Op::Call(code_index) => {
                let block_code = self.code.codes.get(code_index).unwrap().clone();
                self.enter_subroutine(block_code);
            }
        }
    }

    fn lookup_value(&self, name: Ident) -> &Value {
        if let Some(val) = self.scope.get(&name) {
            val
        } else if let Some(parent) = &self.parent {
            parent.0.lookup_value(name)
        } else {
            panic!("Could not find variable {}", name)
        }
    }

    fn lookup_value_mut(&mut self, name: Ident) -> &mut Value {
        if let Some(val) = self.scope.get_mut(&name) {
            val
        } else if let Some(parent) = &mut self.parent {
            parent.0.lookup_value_mut(name)
        } else {
            panic!("Could not find variable {}", name)
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

    fn exit_subroutine(&mut self) {
        let (parent, ptr) = mem::take(&mut self.parent).unwrap();
        let child = mem::replace(self, *parent);
        //self.code = child.code;
        self.stack = child.stack;
        self.op_pointer = ptr;
    }
}
