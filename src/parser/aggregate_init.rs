use crate::lexer::escape_sequences::CharType;
use crate::lexer::language_features::AssignmentTypes;
use crate::lexer::language_features::LiteralTypes;
use crate::lexer::language_features::OperatorTypes;
use crate::lexer::lexer::{Lexer, TokenTypes};
use crate::parser::expression_parser::ExprNode;
use crate::parser::expression_parser::parse_accessor_operator;
use crate::parser::helper::pretty_clean_string;
use crate::parser::parser::InitalizerNode;
use crate::parser::parser::{STOP_AT_COMMA, parse_initalizer};
use std::fmt::Display;

#[derive(Debug, Clone)]
pub enum AggregateInit {
    Aggregate {
        held_values: Vec<AggregateInit>,
    },
    Designator {
        // it can be an expression, it just has to be known at compile time
        // so enum constants or simple expressions (1+2)
        index: ExprNode,
        value: Box<InitalizerNode>,
    },
    InitElement {
        value: Box<InitalizerNode>,
    },
    MemberAccess {
        members: Vec<String>,
        value: Box<InitalizerNode>,
    },
}

impl Display for AggregateInit {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let final_str = self.display(0);

        write!(display, "{final_str}")
    }
}

impl AggregateInit {
    fn display(&self, indentation: usize) -> String {
        let mut output = String::new();
        let indent_str = " ".repeat(indentation);
        let next_indent_str = " ".repeat(indentation + 2);

        match self {
            AggregateInit::InitElement { value } => {
                let value = pretty_clean_string(&value.to_string());

                output.push_str(&format!(
                    "{indent_str}(InitElement\n{next_indent_str}(Expr {value}))"
                ));
            }

            AggregateInit::Designator { index, value } => {
                let value = pretty_clean_string(&value.to_string());
                let index = pretty_clean_string(&index.to_string());
                output.push_str(&format!("{indent_str}(InitElement\n{next_indent_str}(Index {index})\n{next_indent_str}(Expr {value}))"));
            }

            AggregateInit::Aggregate { held_values } => {
                output.push_str(&format!("{indent_str}(AggInit"));

                for value in held_values {
                    output.push_str(&format!("\n{}", value.display(indentation + 2)));
                }

                output.push(')');
            }

            AggregateInit::MemberAccess { members, value } => {
                let value = pretty_clean_string(&value.to_string());

                output.push_str(&format!("{indent_str}(Member\n{next_indent_str}"));

                for member in members {
                    output.push_str(&format!("(Var {member} "));
                }

                output.push_str(&format!("(Expr {value})"));
                output.push_str(&")".repeat(members.len()));
                output.push(')');
            }
        }

        output
    }
}

pub fn parse_aggregate_init(lexer: &mut Lexer) -> Result<AggregateInit, String> {
    if matches!(lexer.peek(), Some(TokenTypes::LCurlyBrace)) {
        lexer.advance();
    }

    lexer.force_peek("Expected next token in aggregate initalization, got nothing")?;

    let mut all_init_elements = Vec::new();

    while let Some(token) = lexer.peek()
        && !matches!(token, TokenTypes::RCurlyBrace)
    {
        match token {
            TokenTypes::Operator(OperatorTypes::LSquareBracket) => {
                all_init_elements.push(parse_designator_element(lexer)?);
            }
            TokenTypes::Operator(OperatorTypes::DotOperator) => {
                all_init_elements.push(parse_aggregate_member(lexer)?);
            }
            TokenTypes::LCurlyBrace => {
                all_init_elements.push(parse_nested_aggregate(lexer)?);
            }
            TokenTypes::RCurlyBrace => {
                all_init_elements.push(AggregateInit::Aggregate {
                    held_values: vec![],
                });
            }
            TokenTypes::Literal(LiteralTypes::String(_)) => {
                all_init_elements.push(parse_string_initalization(lexer)?);
            }

            TokenTypes::Literal(_) => {
                all_init_elements.push(parse_init_element(lexer)?);
            }

            _ => {
                return Err(String::from(format!(
                    "Unexpected token of type {token} in aggregate initalization"
                )));
            }
        };
    }

    lexer.advance(); // move past the "}"

    Ok(AggregateInit::Aggregate {
        held_values: all_init_elements,
    })
}

fn parse_string_initalization(lexer: &mut Lexer) -> Result<AggregateInit, String> {
    let string_literal = lexer.expect_extract(|x| match x {
        TokenTypes::Literal(LiteralTypes::String(str)) => Some(str),
        _ => None,
    })?;

    let mut all_elements = Vec::new();

    for char_type in string_literal.0.iter() {
        let initalizer = InitalizerNode {
            aggregate: None,
            expr: Some(ExprNode::Char {
                character: char_type.clone(),
            }),
        };
        let init_element = AggregateInit::InitElement {
            value: Box::new(initalizer),
        };
        all_elements.push(init_element);
    }

    let null_initalizer = InitalizerNode {
        aggregate: None,
        expr: Some(ExprNode::Char {
            character: CharType::Octal { value: 0 }, // null terminator
        }),
    };

    let null_init_element = AggregateInit::InitElement {
        value: Box::new(null_initalizer),
    };
    all_elements.push(null_init_element);

    let next_token =
        lexer.force_peek("Unexpected end to string parsing in aggregate initalization")?;
    if matches!(next_token, TokenTypes::Operator(OperatorTypes::Comma)) {
        lexer.advance();
    }

    Ok(AggregateInit::Aggregate {
        held_values: all_elements,
    })
}

fn parse_aggregate_member(lexer: &mut Lexer) -> Result<AggregateInit, String> {
    let mut members = Vec::new();
    let init_value;

    loop {
        lexer.expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::DotOperator)))?;
        let member_name = lexer.expect_extract(|x| match x {
            TokenTypes::Identifier(x) => Some(x),
            _ => None,
        })?;

        members.push(member_name);

        let next_token = lexer.force_peek("Expected member accessor or assignment, got nothing")?;

        if let TokenTypes::Assignment(AssignmentTypes::SimpleAssignment) = next_token {
            lexer.advance();

            init_value = parse_initalizer::<STOP_AT_COMMA>(lexer)?;
            break;
        }
    }

    let next_token = lexer.force_peek("Unexpected end to aggregate member initalization")?;
    if !matches!(next_token, TokenTypes::Operator(OperatorTypes::Comma))
        && !matches!(next_token, TokenTypes::RCurlyBrace)
    {
        return Err(format!(
            "Unexpected next token of type {next_token}, expected comma or right curly brace"
        ));
    }

    if matches!(next_token, TokenTypes::Operator(OperatorTypes::Comma)) {
        lexer.advance();
    }

    Ok(AggregateInit::MemberAccess {
        members,
        value: Box::new(init_value),
    })
}

fn parse_nested_aggregate(lexer: &mut Lexer) -> Result<AggregateInit, String> {
    let nested_aggregate = parse_aggregate_init(lexer)?;

    let next_token = lexer.force_peek("Unexpected end to aggregate member initalization")?;

    if matches!(next_token, TokenTypes::Operator(OperatorTypes::Comma)) {
        lexer.advance();
    }

    Ok(nested_aggregate)
}

fn parse_designator_element(lexer: &mut Lexer) -> Result<AggregateInit, String> {
    let index_expr = parse_accessor_operator(lexer)?;

    lexer.expect(|x| matches!(x, TokenTypes::Assignment(AssignmentTypes::SimpleAssignment)))?;

    let initalizer = parse_initalizer::<STOP_AT_COMMA>(lexer)?;

    let designator = AggregateInit::Designator {
        index: index_expr,
        value: Box::new(initalizer),
    };

    let next_token = lexer.force_peek("Expected end of aggregate designator, got nothing")?;

    if matches!(next_token, TokenTypes::Operator(OperatorTypes::Comma)) {
        lexer.advance();
    }

    Ok(designator)
}

fn parse_init_element(lexer: &mut Lexer) -> Result<AggregateInit, String> {
    let initalizer = parse_initalizer::<STOP_AT_COMMA>(lexer)?;
    let aggregate_element = AggregateInit::InitElement {
        value: Box::new(initalizer),
    };

    let next_token = lexer.force_peek("Expected next token in aggregate initalization")?;

    if !matches!(next_token, TokenTypes::Operator(OperatorTypes::Comma))
        && !matches!(next_token, TokenTypes::RCurlyBrace)
    {
        return Err(format!(
            "Unexpected next token of type {next_token}, expected comma or right curly brace"
        ));
    }

    if matches!(next_token, TokenTypes::Operator(OperatorTypes::Comma)) {
        lexer.advance();
    }

    Ok(aggregate_element)
}

#[cfg(test)]
mod tests {
    use crate::parser::aggregate_init::parse_aggregate_init;
    use crate::parser::helper::run_tests;

    #[test]
    fn test_aggregate_init_simple() {
        let test_cases = vec![
            ("{1}", "(AggInit (InitElement (Expr (Num 1))))"),
            (
                "{1, 2}",
                "
                (AggInit 
                    (InitElement (Expr (Num 1)))
                    (InitElement (Expr (Num 2)))
                )
                ",
            ),
            (
                "{1, 2+1}",
                "
                (AggInit
                    (InitElement (Expr (Num 1)))
                    (InitElement (Expr (Binary (Num 2) (Op +) (Num 1))))
                )
                ",
            ),
            (
                "{'h', 'a', 'r', 'r', 'y', '\0'}",
                "
                (AggInit
                    (InitElement (Expr (Char h)))
                    (InitElement (Expr (Char a)))
                    (InitElement (Expr (Char r)))
                    (InitElement (Expr (Char r)))
                    (InitElement (Expr (Char y)))
                    (InitElement (Expr (Char \0)))
                )
                    
                ",
            ),
        ];

        run_tests(parse_aggregate_init, test_cases);
    }

    #[test]
    fn test_aggregate_init_multi_dimensional() {
        let test_cases = vec![
            (
                "{{1, 2, 3}, {4, 5, 6}}",
                "
                (AggInit
                    (AggInit
                        (InitElement (Expr (Num 1)))
                        (InitElement (Expr (Num 2)))
                        (InitElement (Expr (Num 3)))
                    )
                    (AggInit
                        (InitElement (Expr (Num 4)))
                        (InitElement (Expr (Num 5)))
                        (InitElement (Expr (Num 6)))
                    )
                )    
                ",
            ),
            (
                "{{1, 2}, {1, 2, 3}}",
                "
                (AggInit
                    (AggInit
                        (InitElement (Expr (Num 1)))
                        (InitElement (Expr (Num 2)))
                    )
                    (AggInit
                        (InitElement (Expr (Num 1)))
                        (InitElement (Expr (Num 2)))
                        (InitElement (Expr (Num 3)))
                    )
                    
                )                     
                ",
            ),
            (
                "{{1, 2}, 3, 4, 5, 6}",
                "
                (AggInit
                    (AggInit
                        (InitElement (Expr (Num 1)))
                        (InitElement (Expr (Num 2)))
                    )
                    
                    (InitElement (Expr (Num 3)))
                    (InitElement (Expr (Num 4)))
                    (InitElement (Expr (Num 5)))
                    (InitElement (Expr (Num 6)))
                )
                    
                ",
            ),
            (
                "{{1, 2, 3}, {4, 5, 6}, 7, 8, 9}",
                "
                (AggInit
                    (AggInit
                        (InitElement (Expr (Num 1)))
                        (InitElement (Expr (Num 2)))
                        (InitElement (Expr (Num 3)))
                    )
                    (AggInit
                        (InitElement (Expr (Num 4)))
                        (InitElement (Expr (Num 5)))
                        (InitElement (Expr (Num 6)))
                    )
                    (InitElement (Expr (Num 7)))
                    (InitElement (Expr (Num 8)))
                    (InitElement (Expr (Num 9)))
                )

                    
                ",
            ),
            (
                "
                {
                  {
                    { {1, 2}, 3},
                    { 4, 5, 6 }
                  },
                  7, 8, 9
                }
                ",
                "
                (AggInit
                    (AggInit
                        (AggInit
                            (AggInit
                                (InitElement (Expr (Num 1)))
                                (InitElement (Expr (Num 2)))
                            )
                            (InitElement (Expr (Num 3)))
                        )
                        (AggInit
                            (InitElement (Expr (Num 4)))
                            (InitElement (Expr (Num 5)))
                            (InitElement (Expr (Num 6)))
                        )
                    )

                    (InitElement (Expr (Num 7)))
                    (InitElement (Expr (Num 8)))
                    (InitElement (Expr (Num 9)))
                )   
                ",
            ),
        ];

        run_tests(parse_aggregate_init, test_cases);
    }

    #[test]
    fn test_aggregate_init_designators() {
        let test_cases = vec![
            (
                "{[1] = 5}",
                "
                (AggInit
                    (InitElement

                        (Index (Accessor (Num 1)))
                        (Expr (Num 5))
                    )
                )
                ",
            ),
            (
                "{[1] = 5, [1] = 6}",
                "
                (AggInit
                    (InitElement
                        (Index (Accessor (Num 1)))
                        (Expr (Num 5))
                    )
                    (InitElement
                        (Index (Accessor (Num 1)))
                        (Expr (Num 6))
                    )
                )
                ",
            ),
            (
                "{[3] = 7, 8, 9}",
                "
               (AggInit
                    (InitElement
                        (Index (Accessor (Num 3)))
                        (Expr (Num 7))
                    )
                    (InitElement (Expr (Num 8)))
                    (InitElement (Expr (Num 9)))
                )
                ",
            ),
            (
                "{[5] = 1, 2, [2] = 3, 4}",
                "
                (AggInit
                    (InitElement
                        (Index (Accessor (Num 5)))
                        (Expr (Num 1))
                    )
                    (InitElement (Expr (Num 2)))
                    (InitElement
                        (Index (Accessor (Num 2)))
                        (Expr (Num 3))
                    )
                    (InitElement (Expr (Num 4)))
                )

                ",
            ),
            (
                "{[1][2] = 5}",
                "
                (AggInit
                    (InitElement
                        (Index (Accessor (Num 1) (Accessor (Num 2))))
                        (Expr (Num 5))
                    )
                )
                ",
            ),
            (
                "{[1] = { [2] = 9 }}",
                "
                (AggInit
                    (InitElement
                        (Index (Accessor (Num 1)))
                        (Expr 
                            (AggInit
                                (InitElement
                                    (Index (Accessor (Num 2)))
                                    (Expr (Num 9))
                                )
                            )
                        )
                    )
                )

                ",
            ),
        ];

        run_tests(parse_aggregate_init, test_cases);
    }

    #[test]
    fn test_aggregate_init_members() {
        let test_cases = vec![
            (
                "{.x = 20, .y = 10}",
                "
                (AggInit
                    (Member (Var x (Expr (Num 20))))
                    (Member (Var y (Expr (Num 10))))
                )
                ",
            ),
            (
                "{.y = 5, 6}",
                "
                (AggInit
                    (Member (Var y (Expr (Num 5))))
                    (InitElement (Expr (Num 6)))
                )
                ",
            ),
            (
                "{.p.y = 5}",
                "
                (AggInit
                    (Member (Var p (Var y (Expr (Num 5)))))
                )
                ",
            ),
            (
                "{.a.b.x = 42, .a.b.y = 32}",
                "

                (AggInit
                    (Member (Var a (Var b (Var x (Expr (Num 42))))))
                    (Member (Var a (Var b (Var y (Expr (Num 32))))))
                )
                ",
            ),
            (
                "{.p = {0, 5}}",
                "
                (AggInit
                    (Member (Var p (Expr
                        (AggInit
                            (InitElement (Expr (Num 0)))
                            (InitElement (Expr (Num 5)))
                        )
                    )))
                )

                ",
            ),
        ];

        run_tests(parse_aggregate_init, test_cases);
    }

    #[test]
    fn test_aggregate_init_strings() {
        let test_cases = vec![
            (
                "{\"hello\"}",
                "
                (AggInit
                    (AggInit
                        (InitElement (Expr (Char h)))
                        (InitElement (Expr (Char e)))
                        (InitElement (Expr (Char l)))
                        (InitElement (Expr (Char l)))
                        (InitElement (Expr (Char o)))
                        (InitElement (Expr (Char \\0)))
                    )
                )
                                    
                ",
            ),
            (
                "{\"hello\\0\"}",
                "
                (AggInit
                    (AggInit
                        (InitElement (Expr (Char h)))
                        (InitElement (Expr (Char e)))
                        (InitElement (Expr (Char l)))
                        (InitElement (Expr (Char l)))
                        (InitElement (Expr (Char o)))
                        (InitElement (Expr (Char \\0)))
                        (InitElement (Expr (Char \\0)))
                    )
                )
                ",
            ),
            (
                "{{\"hello\"}}",
                "

                (AggInit
                    (AggInit
                        (AggInit
                            (InitElement (Expr (Char h)))
                            (InitElement (Expr (Char e)))
                            (InitElement (Expr (Char l)))
                            (InitElement (Expr (Char l)))
                            (InitElement (Expr (Char o)))
                            (InitElement (Expr (Char \\0)))
                        )
                    )
                )
                ",
            ),
            (
                "{\"abc\", \"xy\"}",
                "
                (AggInit
                    (AggInit
                        (InitElement (Expr (Char a)))
                        (InitElement (Expr (Char b)))
                        (InitElement (Expr (Char c)))
                        (InitElement (Expr (Char \\0)))
                    )
                    (AggInit
                        (InitElement (Expr (Char x)))
                        (InitElement (Expr (Char y)))
                        (InitElement (Expr (Char \\0)))
                    )
                )
                ",
            ),
        ];

        run_tests(parse_aggregate_init, test_cases);
    }

    #[test]
    fn test_aggregate_trailing_commas() {
        let test_cases = vec![
            (
                "{1, 2, 3,}",
                "
                (AggInit
                    (InitElement (Expr (Num 1)))
                    (InitElement (Expr (Num 2)))
                    (InitElement (Expr (Num 3)))
                )
                ",
            ),
            (
                " { { 1, 2, 3, }, { 4, 5, 6, }, }",
                "
                (AggInit
                    (AggInit
                        (InitElement (Expr (Num 1)))
                        (InitElement (Expr (Num 2)))
                        (InitElement (Expr (Num 3)))
                    )
                    (AggInit
                        (InitElement (Expr (Num 4)))
                        (InitElement (Expr (Num 5)))
                        (InitElement (Expr (Num 6)))
                    )
                )
                ",
            ),
            (
                " { { \"Al\", 30, }, { \"Bob\",   40, }, } ",
                "
                (AggInit
                    (AggInit
                        (AggInit
                            (InitElement (Expr (Char A)))
                            (InitElement (Expr (Char l)))
                            (InitElement (Expr (Char \\0)))
                        )

                        (InitElement (Expr (Num 30)))
                    )
                    (AggInit
                        (AggInit
                            (InitElement (Expr (Char B)))
                            (InitElement (Expr (Char o)))
                            (InitElement (Expr (Char b)))
                            (InitElement (Expr (Char \\0)))
                        )

                        (InitElement (Expr (Num 40)))
                    )
                )
                ",
            ),
            (
                " { \"Alice\", 30, \"Bob\",   40,} ",
                "
                (AggInit
                    (AggInit
                        (InitElement (Expr (Char A)))
                        (InitElement (Expr (Char l)))
                        (InitElement (Expr (Char i)))
                        (InitElement (Expr (Char c)))
                        (InitElement (Expr (Char e)))
                        (InitElement (Expr (Char \\0)))
                    )

                    (InitElement (Expr (Num 30)))

                    (AggInit
                        (InitElement (Expr (Char B)))
                        (InitElement (Expr (Char o)))
                        (InitElement (Expr (Char b)))
                        (InitElement (Expr (Char \\0)))
                    )

                    (InitElement (Expr (Num 40)))
                )  
                ",
            ),
            (
                "{ [0] = 1, [5] = 42,}",
                "    
                (AggInit
                    (InitElement
                        (Index (Accessor (Num 0)))
                        (Expr (Num 1))
                    )

                    (InitElement
                        (Index (Accessor (Num 5)))
                        (Expr (Num 42))
                    )
                )
                ",
            ),
        ];

        run_tests(parse_aggregate_init, test_cases);
    }
}
