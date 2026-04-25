use crate::lexer::language_features::{LiteralTypes, OperatorTypes};
use crate::lexer::lexer::{Lexer, TokenTypes};

use std::fmt::Display;

pub enum Node {
    Block { nodes: Vec<Node> },
    Expr { expr: ExprNode },
    // while
    // if
    // declaration
    // etc
}

pub enum ExprNode {
    BinaryNode {
        left: Box<ExprNode>,
        operator: OperatorTypes,
        right: Box<ExprNode>,
    },
    NumberNode {
        num: u64,
    },
    IdentifierNode {
        identifier: String,
    },
}

impl ExprNode {
    fn display(node: &ExprNode, string: &mut String, indentation_level: usize) {
        let indentation = " ".repeat(indentation_level);

        match (node) {
            Self::BinaryNode {
                left,
                operator,
                right,
            } => {
                string.push_str(&format!("{indentation}Binary Node:\n"));

                ExprNode::display(left, string, indentation_level + 2);

                string.push_str(&format!(
                    "  {indentation}Operator: {}\n",
                    operator.to_string()
                ));
                ExprNode::display(right, string, indentation_level + 2);
            }
            Self::NumberNode { num } => {
                string.push_str(&format!("{indentation}Number: {}\n", num.to_string()));
            }

            Self::IdentifierNode { identifier } => {
                string.push_str(&format!("{indentation}Identifier: {identifier}\n"));
            }

            _ => todo!(),
        }
    }
}

impl Display for ExprNode {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut final_str = String::new();
        ExprNode::display(self, &mut final_str, 0);

        write!(display, "{final_str}")
    }
}

// precedence climbing
pub fn parse_expression(lexer: &mut Lexer, min_precedence: u8) -> ExprNode {
    let mut left = parse_primary(lexer).unwrap();

    while let Some(TokenTypes::Operator(operator_type)) = lexer.peek() {
        let precedence = operator_type.precedence();

        if precedence < min_precedence {
            break;
        }

        lexer.advance();
        // todo: dont add when its right associativity
        let next_min_precedence = precedence + 1;

        let right = parse_expression(lexer, next_min_precedence);
        left = ExprNode::BinaryNode {
            left: Box::new(left),
            operator: operator_type,
            right: Box::new(right),
        }
    }

    left
}

fn parse_primary(lexer: &mut Lexer) -> Result<ExprNode, ()> {
    if let Some(token_type) = lexer.next() {
        match token_type {
            TokenTypes::Literal(literal_type) => match literal_type {
                LiteralTypes::Integer(x) => return Ok(ExprNode::NumberNode { num: x }),
                _ => todo!(),
            },

            TokenTypes::Identifier(identifier) => {
                return Ok(ExprNode::IdentifierNode { identifier });
            }

            _ => todo!(),
        }
    }

    Err(())
}
