use crate::{
    lexer::lexer::Lexer,
    parser::{
        aggregate_init::parse_aggregate_init, parser::parse_program, type_parser::parse_type,
    },
};

mod lexer;
mod parser;
mod semantics;

const PROGRAM: &str = r#"


// struct Point;

// struct Outer{
//     int outer_var;
//     struct Inner {
//       int inner_var;
//     } innner;
// };



struct Point p = {1,2}, q = {3,4};

// struct Point{int x; int y;} = ;

"#;

fn main() {
    let mut lexer = Lexer::new(PROGRAM).unwrap();

    println!("{lexer}");
    let expression = parse_program(&mut lexer).unwrap();
    // let expression = parse_type(&mut lexer).unwrap();
    // let expression = parse_aggregate_init(&mut lexer).unwrap();

    println!("{expression}");
}
