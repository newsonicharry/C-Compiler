use crate::{lexer::lexer::Lexer, parser::parser::parse_program};

mod lexer;
mod parser;

const EXPRESSION: &str = "

    int my_function(char* param1, float x);
    int* x = 10;


";

// int* my_func(float x, char y);

fn main() {
    let mut lexer = Lexer::new(EXPRESSION);
    let program = parse_program(&mut lexer).unwrap();
    println!("{}", program);
    // let final_type = parse_type(type_parser::parse_type&mut lexer).unwrap();

    // println!("{}", final_type);
}
