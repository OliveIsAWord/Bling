mod lexing;
mod parsing;

fn main() {
    println!("Hello, Bling 0.2!");
    let program = "({})";
    let tokens = lexing::lex(program).unwrap();
    dbg!(tokens);
    // let ast = parsing::parse(tokens).unwrap();
    // dbg!(ast);
}
