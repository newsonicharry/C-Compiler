use crate::lexer::escape_sequences::CharType;
use crate::lexer::language_features::LiteralTypes;
use crate::lexer::language_features::OperatorTypes;
use crate::lexer::lexer::{Lexer, TokenTypes};
use crate::lexer::number_parser::IntType;
use std::fmt::Display;
use std::fmt::format;

#[derive(Clone, Debug)]
pub enum ExprNode {
    Empty,
    Binary {
        left: Box<ExprNode>,
        operator: OperatorTypes,
        right: Box<ExprNode>,
    },
    Integer {
        num: IntType,
    },
    Identifier {
        identifier: String,
    },
    Char {
        character: CharType,
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
    // PostDec,
    // FunctionCall {
    //     identifier: String,
    //     // args: Vec<ExprNode>,
    // },
    Accessor {
        expr: Box<ExprNode>,
        nested_accessor: Box<ExprNode>,
    },
}

impl ExprNode {
    fn display(self, indent: usize) -> String {
        let mut output = String::new();
        let indent_str = " ".repeat(indent);
        let next_indent_str = " ".repeat(indent + 2);

        match self {
            Self::Binary {
                left,
                operator,
                right,
            } => {
                output.push_str(&format!(
                    "{indent_str}(Binary\n{}\n{next_indent_str}(Op  {operator})\n{})",
                    left.display(indent + 2),
                    right.display(indent + 2)
                ));
            }

            Self::PostFix { left, right } => {
                output.push_str(&format!(
                    "{indent_str}(Postfix\n{}\n{})",
                    left.display(indent + 2),
                    right.display(indent + 2)
                ));
            }

            Self::Unary { operator, expr } => {
                output.push_str(&format!(
                    "{indent_str}(Unary\n{next_indent_str}(Op {operator})\n{})",
                    expr.display(indent + 2)
                ));
            }

            Self::Integer { num } => {
                output.push_str(&format!("{indent_str}(Num {num})"));
            }

            Self::Identifier { identifier } => {
                output.push_str(&format!("{indent_str}(Var {identifier})"));
            }

            Self::Char { character } => {
                output.push_str(&format!("{indent_str}(Char {character})"));
            }

            Self::Accessor {
                expr,
                nested_accessor,
            } => {
                output.push_str(&format!(
                    "{indent_str}(Accessor\n{}",
                    expr.display(indent + 2),
                ));

                if !matches!(*nested_accessor, ExprNode::Empty) {
                    output.push_str(&format!("\n{}", nested_accessor.display(indent + 2)));
                }

                output.push(')');
            }

            Self::PostInc => {
                output.push_str(&format!("{indent_str}++\n"));
            }

            Self::Empty => {
                output.push_str(&format!("{indent_str}(Empty)"));
            }
        }

        output
    }
}

impl Display for ExprNode {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let final_str = self.clone().display(0);

        write!(display, "{final_str}")
    }
}

pub fn parse_unary(lexer: &mut Lexer) -> Result<ExprNode, String> {
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

pub fn parse_accessor_operator(lexer: &mut Lexer) -> Result<ExprNode, String> {
    let mut expr_nodes = Vec::new();

    while let Some(token) = lexer.peek() {
        if !matches!(token, TokenTypes::Operator(OperatorTypes::LSquareBracket)) {
            break;
        }

        lexer.advance();

        expr_nodes.push(parse_expression(lexer, 0)?);

        lexer.expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::RSquareBracket)))?;
    }

    let mut final_accessor = ExprNode::Empty;

    // we reverse it because when building the tree
    // we have to start at the most inner most nested part
    for node in expr_nodes.iter().rev() {
        final_accessor = ExprNode::Accessor {
            expr: Box::new(node.clone()),
            nested_accessor: Box::new(final_accessor.clone()),
        };
    }

    Ok(final_accessor)
}

fn parse_postfix(lexer: &mut Lexer) -> Result<ExprNode, String> {
    let mut node = ExprNode::Empty;

    if let Some(TokenTypes::Identifier(identifier)) = lexer.peek() {
        node = ExprNode::Identifier { identifier };
        lexer.advance();
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
            OperatorTypes::LParen => {
                lexer.advance();
                node = parse_expression(lexer, 0)?;
                lexer.advance();
            }
            OperatorTypes::LSquareBracket => {
                node = ExprNode::PostFix {
                    left: Box::new(node),
                    right: Box::new(parse_accessor_operator(lexer)?),
                };
            }
            // OperatorTypes::ArrowOperator => {}
            // OperatorTypes::DotOperator => {}
            _ => break,
        }
    }

    return Ok(node);
}

pub fn parse_expression(lexer: &mut Lexer, min_precedence: u8) -> Result<ExprNode, String> {
    let mut left = parse_primary(lexer)?;

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

        let right = parse_expression(lexer, next_min_precedence)?;
        left = ExprNode::Binary {
            left: Box::new(left),
            operator: operator_type,
            right: Box::new(right),
        };
    }

    Ok(left)
}

fn parse_primary(lexer: &mut Lexer) -> Result<ExprNode, String> {
    if let Some(token_type) = lexer.peek() {
        match token_type {
            TokenTypes::Literal(literal_type) => match literal_type {
                LiteralTypes::Integer(x) => {
                    lexer.advance();
                    return Ok(ExprNode::Integer { num: x });
                }
                LiteralTypes::Character(x) => {
                    lexer.advance();
                    return Ok(ExprNode::Char { character: (x) });
                }

                _ => todo!(),
            },

            TokenTypes::Identifier(_) => return parse_postfix(lexer),
            TokenTypes::Operator(_) => return parse_unary(lexer),

            _ => {
                return Err(format!(
                    "Expected primary token in expression parser, got token of type {token_type}"
                ));
            }
        }
    }

    if lexer.peek().is_none() {
        return Err(String::from("Expected another token, got nothing"));
    }

    Err(String::from(
        "Next token must be a literal, operator or identifier",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::expression_parser::{ExprNode, parse_expression};
    use crate::parser::helper::run_tests;

    fn parse_expression_generic(lexer: &mut Lexer) -> Result<ExprNode, String> {
        parse_expression(lexer, 0)
    }

    #[test]
    fn expression_primary() {
        let test_cases = vec![
            (r#"x"#, "(Var x)"),
            (r#"a123"#, "(Var a123)"),
            (r#"123"#, "(Num 123)"),
            (r#"0"#, "(Num 0)"),
            (r#"077"#, "(Num 63)"), // this is octal
            (r#"0xFF"#, "(Num 255)"),
            // (r#"3.14"#, ""),
            // (r#"1e10"#, ""),
            // (r#"'a'"#, ""),
            // (r#"'\n'"#, ""),
            // (r#""hello""#, ""),
            // (r#"(a)"#, ""),
            // (r#"((a))"#, ""),
            // (r#"(((a)))"#, ""),
        ];

        run_tests(parse_expression_generic, test_cases);
    }
}
