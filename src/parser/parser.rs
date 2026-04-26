use crate::lexer;
use crate::lexer::language_features::{LiteralTypes, OperatorTypes};
use crate::lexer::lexer::{Lexer, TokenTypes};

use std::collections::binary_heap::PeekMut;
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
    Binary {
        left: Box<ExprNode>,
        operator: OperatorTypes,
        right: Box<ExprNode>,
    },
    Number {
        num: u64,
    },
    Identifier {
        identifier: String,
    },

    Unary {
        operator: OperatorTypes,
        expr: Box<ExprNode>,
    },
    PostFix {
        left: Box<ExprNode>,
        right: Box<ExprNode>,
    },

    PostInc,
    PostDec,
    FunctionCall {
        identifier: String,
        // args: Vec<ExprNode>,
    },
    Accessor {
        expr: Box<ExprNode>,
    },
}

impl ExprNode {
    fn display(node: &ExprNode, string: &mut String, indentation_level: usize) {
        let indentation = " ".repeat(indentation_level);

        match node {
            Self::Binary {
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

            Self::PostFix { left, right } => {
                string.push_str(&format!("{indentation}Postfix Node:\n"));

                ExprNode::display(left, string, indentation_level + 2);
                ExprNode::display(right, string, indentation_level + 2);
            }

            Self::Unary { operator, expr } => {
                string.push_str(&format!("{indentation}Unary Node:\n"));

                string.push_str(&format!(
                    "  {indentation}Operator: {}\n",
                    operator.to_string()
                ));

                ExprNode::display(expr, string, indentation_level + 2);
            }

            Self::Number { num } => {
                string.push_str(&format!("{indentation}Number: {}\n", num.to_string()));
            }

            Self::Identifier { identifier } => {
                string.push_str(&format!("{indentation}Identifier: {identifier}\n"));
            }

            Self::Accessor { expr } => {
                string.push_str(&format!("{indentation}Accessor:\n"));

                ExprNode::display(expr, string, indentation_level + 2);
            }

            Self::PostInc => {
                string.push_str(&format!("{indentation}Operator: PostInc\n"));
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

pub fn parse_unary(lexer: &mut Lexer) -> Result<ExprNode, ()> {
    if let Some(TokenTypes::Operator(operator_type)) = lexer.peek()
        && operator_type.potential_unary()
    {
        lexer.advance();
        let operand = parse_unary(lexer);
        return Ok(ExprNode::Unary {
            operator: operator_type,
            expr: Box::new(operand?),
        });
    }

    return parse_postfix(lexer);
}

fn parse_postfix(lexer: &mut Lexer) -> Result<ExprNode, ()> {
    let mut node: ExprNode;

    if let Some(TokenTypes::Identifier(identifier)) = lexer.peek() {
        node = ExprNode::Identifier { identifier };
        lexer.advance();
        // node
    } else {
        return Err(());
    }

    while let Some(TokenTypes::Operator(operator_type)) = lexer.peek() {
        match operator_type {
            OperatorTypes::Inc => {
                node = ExprNode::PostFix {
                    left: Box::new(node),
                    right: Box::new(ExprNode::PostInc),
                };
                lexer.advance();
            }
            // OperatorTypes::Dec => node = Some(ExprNode::PostDec),
            // OperatorTypes::LParen => {}
            OperatorTypes::LSquareBracket => {
                lexer.advance();
                node = ExprNode::PostFix {
                    left: Box::new(node),
                    right: Box::new(ExprNode::Accessor {
                        expr: Box::new(parse_expression(lexer, 0)),
                    }),
                };

                lexer.expect(TokenTypes::Operator(OperatorTypes::RSquareBracket))?;
            }
            // OperatorTypes::ArrowOperator => {}
            // OperatorTypes::DotOperator => {}
            _ => break,
        }
    }

    return Ok(node);
}

pub fn parse_expression(lexer: &mut Lexer, min_precedence: u8) -> ExprNode {
    let mut left = parse_primary(lexer).unwrap();

    while let Some(TokenTypes::Operator(operator_type)) = lexer.peek()
        && !operator_type.potential_postfix()
    {
        let precedence = operator_type.precedence();

        if precedence < min_precedence {
            break;
        }

        lexer.advance();
        // todo: dont add when its right associativity
        let next_min_precedence = precedence + 1;

        let right = parse_expression(lexer, next_min_precedence);
        left = ExprNode::Binary {
            left: Box::new(left),
            operator: operator_type,
            right: Box::new(right),
        };
    }

    left
}

fn parse_primary(lexer: &mut Lexer) -> Result<ExprNode, ()> {
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

    Err(())
}
