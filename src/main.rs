//mod compile;
mod parse;
//mod little;

use parse::parse;

use std::fs;

// fn remove_whitespace(s: &str) -> String {
//     s.split_whitespace().collect()
// }

fn main() {
    let source = fs::read_to_string("examples/hello_world.bli").unwrap();
    //let _ = source;
    //let source = "";
    println!("{:?}", parse(&source));
}
