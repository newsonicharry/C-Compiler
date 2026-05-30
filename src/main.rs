use crate::{lexer::lexer::Lexer, parser::type_parser::parse_type};

mod lexer;
mod parser;

const EXPRESSION: &str = "const char * const (*(*complex_func)(int, void (*)(int)))(double);";

fn main() {
    let mut lexer = Lexer::new(EXPRESSION);
    let final_type = parse_type(&mut lexer).unwrap();

    println!("{}", final_type);
}
