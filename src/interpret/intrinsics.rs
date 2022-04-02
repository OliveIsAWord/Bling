use super::macros::double_try;
use super::{ExecResult, Executor, ScriptError, Value};
use num_bigint::BigInt;

pub fn print(exec: &mut Executor) -> ExecResult<Value> {
    let val = exec.pop_stack()?;
    println!("{:?}", val);
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

arithmetic_intrinsic! {add, |x, y| Value::Number(x + y)}
arithmetic_intrinsic! {sub, |x, y| Value::Number(x - y)}
arithmetic_intrinsic! {mul, |x, y| Value::Number(x * y)}
arithmetic_intrinsic! {div,
    |x: BigInt, y: BigInt| x.checked_div(&y).map_or(Value::None, Value::Number)
}
