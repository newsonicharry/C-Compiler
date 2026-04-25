use crate::lexer::lexer::Lexer;
use crate::parser::parser::parse_expression;

mod lexer;
mod parser;

// const EXPRESSION: &str = r#"

// int main() {
//     int num1, num2, sum;

//     printf("Enter two integers:");
//     scanf("%d %d", &num1, &num2);

//     sum = num1 + num2;
//     printf("Sum: %d\n" , sum);

//     return 0;
// }

// "#;

const EXPRESSION: &str = "2 * 10 + 12 / num";

fn main() {
    let mut lexer = Lexer::new(EXPRESSION);
    println!("{lexer}");

    let nodes = parse_expression(&mut lexer, 0);
    println!("{}", nodes);
}
