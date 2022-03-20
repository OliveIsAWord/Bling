mod compile;
mod interpret;
mod parse;

use compile::compile_expr;
use interpret::Executor;
use parse::parse;

use std::fs;

fn main() {
    let source = fs::read_to_string("examples/hello_world.bli").unwrap();
    let ast = parse(&source);
    println!("AST ->\n    {:?}", ast);
    let bytecode = compile_expr(ast.unwrap()[0].clone());
    println!("BYTECODE ->\n    {:?}", bytecode);
    let mut exec = Executor::from_code(bytecode);
    println!("INITIAL EXECUTOR ->\n    {:?}", exec);
    exec.run();
    println!("FINISHED EXECUTOR ->\n    {:?}", exec);
}
