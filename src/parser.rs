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
