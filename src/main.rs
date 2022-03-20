mod compile;
mod parse;
//mod little;

use compile::compile_expr;
use parse::parse;

use std::fs;

// fn remove_whitespace(s: &str) -> String {
//     s.split_whitespace().collect()
// }

fn main() {
    let source = fs::read_to_string("examples/hello_world.bli").unwrap();
    //let _ = source;
    //let source = "";
    let ast = parse(&source);
    println!("{:?}", ast);
    let bytecode = compile_expr(ast.unwrap()[0].clone());
    println!("{:?}", bytecode);
}
