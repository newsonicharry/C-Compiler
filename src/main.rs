use crate::{
    lexer::{escape_sequences::split_string, lexer::Lexer, number_parser::parse_number_literal},
    parser::{
        aggregate_init::parse_aggregate_init,
        expression_parser::parse_expression,
        parser::parse_program,
        type_parser::{is_valid_var_name, parse_parameter_list, parse_type},
    },
};

mod lexer;
mod parser;
mod semantics;

const EXPRESSION: &str = r#"


0xABC.DEFp10


"#;

fn main() {
    let mut lexer = Lexer::new(EXPRESSION).unwrap();

    println!("{lexer}");

    // let expression = parse_expression(&mut lexer, 0).unwrap();

    // println!("{expression}");
}
