//! An interpreter for the Bling programming language.

mod compile;
mod interpret;
mod parse;

use compile::compile;
use interpret::Executor;
use parse::parse;

use std::fs;

fn main() {
    let source = fs::read_to_string("examples/lambda.bli").unwrap();
    let ast = parse(&source);
    println!("AST ->\n    {:?}", ast);
    let bytecode = compile(ast.unwrap());
    println!("BYTECODE ->\n    {:?}", bytecode);
    let mut exec = Executor::from_code(bytecode);
    println!("INITIAL EXECUTOR ->\n    {:?}", exec);
    let result = exec.run();
    println!("FINISHED EXECUTOR ->\n    {:?}", exec);
    println!("RESULT ->\n    {:?}", result);
}
