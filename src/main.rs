//! An interpreter for the Bling programming language.

mod compile;
mod interpret;
mod parse;
mod sysexits;

use compile::compile;
use interpret::Executor;
use parse::parse;

use std::env;
use std::fs;
use std::process::exit;

fn main() {
    let args: Vec<String> = env::args().collect();
    let fp = args.get(1).unwrap_or_else(|| {
        // identity closure necessary for converting &String to &str
        let app_name = args.get(0).map_or("bling", |x| x);
        eprintln!("Error: no source file specified");
        eprintln!("Usage: {} <source file>", app_name);
        exit(sysexits::USAGE);
    });
    let source = fs::read_to_string(fp).unwrap_or_else(|e| {
        eprintln!("{}", e);
        exit(sysexits::NO_INPUT);
    });
    let ast = parse(&source).unwrap_or_else(|e| {
        eprintln!("{}", e);
        exit(sysexits::DATA_ERR);
    });
    //println!("AST ->\n    {:?}", ast);
    let (bytecode, idents) = compile(ast);
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
