//! An interpreter for the Bling programming language.

mod compile;
mod interpret;
mod parse;

use compile::compile;
use interpret::Executor;
use parse::parse;

use std::fs;

fn main() {
    let source = fs::read_to_string("examples/maths.bli").unwrap();
    let ast = parse(&source);
    //println!("AST ->\n    {:?}", ast);
    let (bytecode, idents) = compile(ast.unwrap());
    //println!("BYTECODE ->\n    {:?}", bytecode);
    let mut exec = Executor::from_code(bytecode, idents);
    exec.initialize_builtins();
    //println!("INITIAL EXECUTOR ->\n    {:?}", exec);
    println!("\n=== OUTPUT ===");
    let result = exec.run();
    println!("==============\n");
    //println!("FINISHED EXECUTOR ->\n    {:?}", exec);
    println!("RESULT ->\n    {:?}", result);
}
