use crate::{lexer::lexer::Lexer, parser::parser::parse_program};

mod lexer;
mod parser;
mod semantics;

const PROGRAM: &str = r#"
int main(){
    if (x == 0)
        a();
    else if (x == 1)
        b();
    else if (x == 2)
        c();
    else
        d();
}       

"#;

fn main() {
    let mut lexer = Lexer::new(PROGRAM).unwrap();

    println!("{lexer}");

    let expression = parse_program(&mut lexer).unwrap();

    println!("{expression}");
}
