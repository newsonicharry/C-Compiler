use crate::lexer::Lexer;
// use crate::parser::parse;

mod lexer;
mod parser;

const EXPRESSION: &str = r#"

int main() {
    int num1, num2, sum;

    printf("Enter two integers:");
    scanf("%d %d", &num1, &num2);

    sum = num1 + num2;
    printf("Sum: %d\n" , sum);

    return 0;
}

"#;

// const EXPRESSION: &str = r#""hello""hi""#;

fn main() {
    let mut lexer = Lexer::new(EXPRESSION);
    println!("{lexer}");

    // let nodes = parse(&mut lexer);
    // println!("{:?}", nodes);
}
