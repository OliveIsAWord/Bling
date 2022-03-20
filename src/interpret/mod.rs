use crate::compile::{Code, Op, Value};
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Executor {
    code: Code,
    scope: HashMap<usize, Value>,
    stack: Vec<Value>,
    // parent: Option<Box<Code>>,
}

impl Executor {
    pub fn from_code(code: Code) -> Self {
        Self {
            code,
            ..Default::default()
        }
    }

    pub fn run(&mut self) {
        for op in &self.code.ops {
            match *op {
                Op::GetConstant(val_index) => {
                    let val = self.code.constants.get(val_index).unwrap();
                    self.stack.push(val.clone());
                }
                Op::GetIdent(ident_index) => {
                    let val = self.scope.get(&ident_index).unwrap();
                    self.stack.push(val.clone());
                }
                // Op::Dup => {
                //     let val = self.stack.last().unwrap().clone();
                //     self.stack.push(val);
                // }
                Op::Declare(ident_index) => {
                    let value = self.stack.pop().unwrap();
                    match self.scope.entry(ident_index) {
                        Entry::Occupied(entry) => {
                            let ident_name = self.code.idents.get(ident_index).unwrap();
                            let existing_value = entry.get();
                            panic!(
                                "Attempted to declare variable {} with value {:?} but was already assigned with value of {:?}",
                                ident_name,
                                value,
                                existing_value,
                            );
                        }
                        Entry::Vacant(space) => {
                            space.insert(value);
                        }
                    }
                }
                Op::Assign(ident_index) => {
                    let value = self.stack.pop().unwrap();
                    match self.scope.entry(ident_index) {
                        Entry::Occupied(mut entry) => {
                            entry.insert(value);
                        }
                        Entry::Vacant(_) => {
                            let ident_name = self.code.idents.get(ident_index).unwrap();
                            panic!(
                                "Attempted to assign variable {} before it was declared",
                                ident_name,
                            );
                        }
                    }
                }
            }
        }
    }
}
