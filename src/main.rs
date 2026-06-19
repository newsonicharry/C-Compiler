use crate::{
    lexer::{escape_sequences::split_string, lexer::Lexer},
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


{"abc", "xy"}
    
"#;

fn main() {
    let mut lexer = Lexer::new(EXPRESSION).unwrap();
    let aggregate = parse_aggregate_init(&mut lexer).unwrap();

    println!("{aggregate}");
}
