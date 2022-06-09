use super::macros::double_try;
use super::{ExecResult, Executor, ScriptError, Value};
use crate::compile::TinyInt;
//use num_traits::{Signed, Zero};

pub fn print(exec: &mut Executor) -> ExecResult<Value> {
    let val = exec.pop_stack()?;
    println!("{}", val);
    Ok(Ok(Value::None))
}

pub fn while_loop(exec: &mut Executor) -> ExecResult<Value> {
    let val2 = exec.pop_stack()?;
    let val1 = exec.pop_stack()?;
    let op_result = match (val1, val2) {
        (Value::Bytecode(condition, 0), Value::Bytecode(body, 0)) => {
            let mut output = Value::None;
            loop {
                let cond_value = double_try!(exec.run_code_object(condition.clone()));
                if !cond_value.truthiness() {
                    break;
                }
                output = double_try!(exec.run_code_object(body.clone()));
            }
            Ok(output)
        }
        _ => Err(ScriptError::ArgumentType),
    };
    Ok(op_result)
}

macro_rules! arithmetic_intrinsic {
    ($self:ident, $oper:expr) => {
        pub fn $self(exec: &mut Executor) -> ExecResult<Value> {
            let val2 = exec.pop_stack()?;
            let val1 = exec.pop_stack()?;
            let op_result = match (val1, val2) {
                (Value::Number(x), Value::Number(y)) => Ok($oper(x, y)),
                _ => Err(ScriptError::ArgumentType),
            };
            Ok(op_result)
        }
    };
}

fn checked_rem_euclid(lhs: TinyInt, rhs: TinyInt) -> Option<TinyInt> {
    if rhs.is_zero() {
        return None;
    }
    let r = lhs % rhs.clone();
    Some(if r.is_negative() ^ rhs.is_negative() {
        r + rhs
    } else {
        r
    })
}

#[test]
fn euclidian() {
    use TinyInt::Inline;
    assert_eq!(
        checked_rem_euclid(Inline(13), Inline(10)).unwrap(),
        Inline(3)
    );
    assert_eq!(
        checked_rem_euclid(Inline(-13), Inline(10)).unwrap(),
        Inline(7)
    );
    assert_eq!(
        checked_rem_euclid(Inline(-13), Inline(-10)).unwrap(),
        Inline(-3)
    );
    assert_eq!(
        checked_rem_euclid(Inline(13), Inline(-10)).unwrap(),
        Inline(-7)
    );
}

arithmetic_intrinsic! {add, |x, y| Value::Number(x + y)}
arithmetic_intrinsic! {sub, |x, y| Value::Number(x - y)}
arithmetic_intrinsic! {mul, |x, y| Value::Number(x * y)}
arithmetic_intrinsic! {div,
    |x: TinyInt, y: TinyInt| x.checked_div(&y).map_or(Value::None, Value::Number)
}
arithmetic_intrinsic! {modulo,
    |x: TinyInt, y: TinyInt| checked_rem_euclid(x, y).map_or(Value::None, Value::Number)
}

#[allow(clippy::unnecessary_wraps)]
pub fn list(_exec: &mut Executor) -> ExecResult<Value> {
    Ok(Ok(Value::List(vec![])))
}

// macro_rules! list_intrinsic {
//     ($self:ident, $oper:expr) => {
//         pub fn $self(exec: &mut Executor) -> ExecResult<Value> {
//             let val2 = exec.pop_stack()?;
//             let val1 = exec.pop_stack()?;
//             let op_result = match (val1, val2) {
//                 (Value::Number(x), Value::Number(y)) => Ok($oper(x, y)),
//                 _ => Err(ScriptError::ArgumentType),
//             };
//             Ok(op_result)
//         }
//     };
// }
//
// pub fn impl_list<F, G>(f: F) -> G
// where F: Fn(Vec<Value>) -> ExecResult<Value>
// where G: Fn(&mut Executor) -> ExecResult<Value>
// {
//     |exec| {
//
//     }
// }

pub fn last(exec: &mut Executor) -> ExecResult<Value> {
    if let Value::List(mut list) = exec.pop_stack()? {
        Ok(list.pop().ok_or(ScriptError::ArgumentValue))
    } else {
        Ok(Err(ScriptError::ArgumentType))
    }
}

pub fn push(exec: &mut Executor) -> ExecResult<Value> {
    let val2 = exec.pop_stack()?;
    let val1 = exec.pop_stack()?;
    if let Value::List(mut list) = val1 {
        list.push(val2);
        Ok(Ok(Value::List(list)))
    } else {
        Ok(Err(ScriptError::ArgumentType))
    }
}

pub fn len(exec: &mut Executor) -> ExecResult<Value> {
    if let Value::List(list) = exec.pop_stack()? {
        Ok(Ok(Value::Number(list.len().into())))
    } else {
        Ok(Err(ScriptError::ArgumentType))
    }
}

pub fn map(exec: &mut Executor) -> ExecResult<Value> {
    let val2 = exec.pop_stack()?;
    let val1 = exec.pop_stack()?;
    if let Value::Bytecode(code, 1) = val1 {
        if let Value::List(list) = val2 {
            let mut results = Vec::with_capacity(list.len());
            for item in list {
                exec.stack.push(item);
                let mapped_item = double_try!(exec.run_code_object(code.clone()));
                results.push(mapped_item);
            }
            Ok(Ok(Value::List(results)))
        } else {
            Ok(Err(ScriptError::ArgumentType))
        }
    } else {
        Ok(Err(ScriptError::ArgumentType))
    }
}

pub fn fold(exec: &mut Executor) -> ExecResult<Value> {
    let val2 = exec.pop_stack()?;
    let val1 = exec.pop_stack()?;
    if let Value::Bytecode(code, 2) = val1 {
        if let Value::List(mut list) = val2 {
            let mut accum = match list.pop() {
                Some(v) => v,
                None => return Ok(Ok(Value::None)),
            };
            for item in list.into_iter().rev() {
                exec.stack.push(item);
                exec.stack.push(accum.clone());
                accum = double_try!(exec.run_code_object(code.clone()));
            }
            Ok(Ok(accum))
        } else {
            Ok(Err(ScriptError::ArgumentType))
        }
    } else {
        Ok(Err(ScriptError::ArgumentType))
    }
}

pub fn filter(exec: &mut Executor) -> ExecResult<Value> {
    let val2 = exec.pop_stack()?;
    let val1 = exec.pop_stack()?;
    if let Value::Bytecode(code, 1) = val1 {
        if let Value::List(list) = val2 {
            let mut results = vec![];
            for item in list {
                exec.stack.push(item.clone());
                if double_try!(exec.run_code_object(code.clone())).truthiness() {
                    results.push(item);
                }
            }
            Ok(Ok(Value::List(results)))
        } else {
            Ok(Err(ScriptError::ArgumentType))
        }
    } else {
        Ok(Err(ScriptError::ArgumentType))
    }
}

pub fn zip(exec: &mut Executor) -> ExecResult<Value> {
    let val2 = exec.pop_stack()?;
    let val1 = exec.pop_stack()?;
    if let Value::List(list1) = val1 {
        if let Value::List(list2) = val2 {
            Ok(Ok(Value::List(
                list1
                    .into_iter()
                    .zip(list2.into_iter())
                    .map(|(a, b)| Value::List(vec![a, b]))
                    .collect(),
            )))
        } else {
            Ok(Err(ScriptError::ArgumentType))
        }
    } else {
        Ok(Err(ScriptError::ArgumentType))
    }
}

pub fn at(exec: &mut Executor) -> ExecResult<Value> {
    let val2 = exec.pop_stack()?;
    let val1 = exec.pop_stack()?;
    if let Value::List(list) = val1 {
        if let Value::Number(n) = val2 {
            let none = Ok(Ok(Value::None));
            let index = if n.is_negative() {
                match (-n).try_into() {
                    Ok(neg_index) => match list.len().checked_sub(neg_index) {
                        Some(index) => index,
                        None => return none,
                    },
                    Err(_) => return none,
                }
            } else if let Ok(index) = n.try_into() {
                index
            } else {
                return none;
            };
            Ok(Ok(list.get(index).cloned().unwrap_or(Value::None)))
        } else {
            Ok(Err(ScriptError::ArgumentType))
        }
    } else {
        Ok(Err(ScriptError::ArgumentType))
    }
}
