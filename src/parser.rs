use crate::lexer::{Lexer, OperatorTypes, TokenTypes};

#[derive(Debug)]
pub enum Node {
    BinaryNode {
        left: Box<Node>,
        operator: OperatorTypes,
        right: Box<Node>,
    },
    NumberNode {
        num: i64,
    },
}

fn parse_add(lexer: &mut Lexer) -> Node {
    let mut left = parse_primary(lexer);

    while lexer.peek().is_some()
        && lexer.peek().unwrap() == TokenTypes::Operator(OperatorTypes::Plus)
    {
        lexer.advance();
        let right = parse_primary(lexer);
        left = Node::BinaryNode {
            left: Box::new(left),
            operator: OperatorTypes::Plus,
            right: Box::new(right),
        }
    }

    return left;
}

fn parse_primary(lexer: &mut Lexer) -> Node {
    let token = lexer.peek();

    match token {
        Some(TokenTypes::Number(x)) => {
            lexer.advance();
            return Node::NumberNode { num: x };
        }
        None | _ => panic!("Bad expression"),
    }
}

pub fn parse(lexer: &mut Lexer) -> Node {
    parse_add(lexer)
}
