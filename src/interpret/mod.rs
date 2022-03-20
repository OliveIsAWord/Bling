use crate::compile::{Code, Op, Value};
use crate::parse::Ident;
use std::collections::hash_map::Entry;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Executor {
    code: Code,
    scope: HashMap<Ident, Value>,
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
            match op {
                Op::GetConstant(val_index) => {
                    let val = self.code.constants.get(*val_index).unwrap();
                    self.stack.push(val.clone());
                }
                Op::Declare(ident_index) => {
                    let ident = self.code.idents.get(*ident_index).unwrap();
                    let value = self.stack.pop().unwrap();
                    match self.scope.entry(ident.to_string()) {
                        Entry::Occupied(entry) => {
                            panic!("Attempted to declare variable {} with value {:?} but was already assigned value of {:?}", ident, value, entry.get());
                        }
                        Entry::Vacant(space) => {
                            space.insert(value);
                        }
                    }
                }
            }
        }
    }
}
