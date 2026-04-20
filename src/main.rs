use crate::lexer::Lexer;
use crate::parser::parse;

mod lexer;
mod parser;

const EXPRESSION: &str = "1+2+3";

fn main() {
    let mut lexer = Lexer::new(EXPRESSION);
    println!("{lexer}");
    let nodes = parse(&mut lexer);
    println!("{:?}", nodes);
}
