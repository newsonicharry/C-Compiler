use crate::lexer::{Lexer, LiteralTypes, OperatorTypes, TokenTypes};
use std::fmt::Display;

#[derive(Debug)]
pub enum Node {
    BinaryNode {
        left: Box<Node>,
        operator: OperatorTypes,
        right: Box<Node>,
    },
    NumberNode {
        num: u64,
    },
    IdentifierNode {
        identifier: String,
    },
}

impl Node {
    fn display(node: &Node, string: &mut String, indentation_level: usize) {
        let indentation = " ".repeat(indentation_level);

        match (node) {
            Self::BinaryNode {
                left,
                operator,
                right,
            } => {
                string.push_str(&format!("{indentation}Binary Node:\n"));

                Node::display(left, string, indentation_level + 2);

                string.push_str(&format!(
                    "  {indentation}Operator: {}\n",
                    operator.to_string()
                ));
                Node::display(right, string, indentation_level + 2);
            }
            Self::NumberNode { num } => {
                string.push_str(&format!("{indentation}Number: {}\n", num.to_string()));
            }

            Self::IdentifierNode { identifier } => {
                string.push_str(&format!("{indentation}Identifier: {identifier}\n"));
            }
        }
    }
}

impl Display for Node {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut final_str = String::new();
        Node::display(self, &mut final_str, 0);

        write!(display, "{final_str}")
    }
}

// precedence climbing
pub fn parse_expression(lexer: &mut Lexer, min_precedence: u8) -> Node {
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
        left = Node::BinaryNode {
            left: Box::new(left),
            operator: operator_type,
            right: Box::new(right),
        }
    }

    left
}

fn parse_primary(lexer: &mut Lexer) -> Result<Node, ()> {
    if let Some(token_type) = lexer.next() {
        match token_type {
            TokenTypes::Literal(literal_type) => match literal_type {
                LiteralTypes::Integer(x) => return Ok(Node::NumberNode { num: x }),
                _ => todo!(),
            },

            TokenTypes::Identifier(identifier) => return Ok(Node::IdentifierNode { identifier }),

            _ => todo!(),
        }
    }

    Err(())
}
