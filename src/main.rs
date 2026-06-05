use crate::{
    lexer::lexer::Lexer,
    parser::{parser::parse_program, type_parser::is_valid_var_name},
};

mod lexer;
mod parser;
mod semantics;

const EXPRESSION: &str = "

struct MyStruct{
  int x;
  float y;
} first, second, third;




";

// int* my_func(float x, char y);

fn main() {
    let mut lexer = Lexer::new(EXPRESSION);
    let program = parse_program(&mut lexer).unwrap();
    println!("{}", program);
    // let final_type = parse_type(type_parser::parse_type&mut lexer).unwrap();

    // println!("{}", final_type);
}
