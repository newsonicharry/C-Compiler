use crate::lexer::language_features::{KeywordTypes, LiteralTypes};
use crate::lexer::lexer::{Lexer, TokenTypes};
use crate::parser::expression_parser::{ExprNode, parse_postfix, parse_unary};

pub enum Node {
    Block {
        nodes: Vec<Node>,
    },
    Expr {
        expr: ExprNode,
    },
    FunctionDeclaration {
        name: String,
        modifiers: Vec<KeywordTypes>,
    },
    FunctionDefinition {
        body: Box<Node>,      // should be a Node::Block
        signature: Box<Node>, // should be a Node::FunctionDeclaration
    },
    // while
    // if
    // declaration
    // etc
}

pub fn parse_program(lexer: &mut Lexer) -> Result<Node, ()> {
    let program = Node::Block { nodes: Vec::new() };

    while let Some(token_type) = lexer.peek() {
        match token_type {
            TokenTypes::Keyword(_) => {}

            _ => return Err(()),
        }
    }

    Ok(program)
}

// char* (*(*foo[5])(char*))[];

pub fn parse_primary(lexer: &mut Lexer) -> Result<ExprNode, String> {
    if let Some(token_type) = lexer.peek() {
        match token_type {
            TokenTypes::Literal(literal_type) => match literal_type {
                LiteralTypes::Integer(x) => {
                    lexer.advance();
                    return Ok(ExprNode::Number { num: x });
                }
                _ => todo!(),
            },

            TokenTypes::Identifier(_) => return parse_postfix(lexer),
            TokenTypes::Operator(_) => return parse_unary(lexer),

            _ => todo!(),
        }
    }

    if lexer.peek().is_none() {
        return Err(String::from("Expected another token, got nothing"));
    }

    Err(String::from(
        "Next token must be a literal, operator or identifier",
    ))
}
