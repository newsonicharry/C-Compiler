use crate::{
    lexer::lexer::Lexer,
    parser::expression_parser::{SizeOf, parse_expression},
};

mod lexer;
mod parser;
mod semantics;

const EXPRESSION: &str = r#"


"#;

fn main() {
    let mut lexer = Lexer::new(EXPRESSION).unwrap();

    println!("{lexer}");
    let expression = parse_expression(&mut lexer, 0).unwrap();

    println!("{expression}");
}
