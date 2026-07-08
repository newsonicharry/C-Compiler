use crate::lexer::language_features::{KeywordTypes, OperatorTypes};
use crate::lexer::lexer::{Lexer, TokenTypes};
use crate::parser::nodes::{GlobalNode, StatementNode};
use crate::parser::parser::Parser;
use crate::semantics::semantics::Semantics;
use std::fmt::Display;

pub fn pretty_clean_string(string: &str) -> String {
    let sections: Vec<&str> = string.split_whitespace().collect();
    let result = sections.join(" ");

    result.chars().filter(|x| *x != '\n').collect()
}

pub fn raw_clean_string(string: &str) -> String {
    string
        .chars()
        .filter(|x| *x != '\t' && *x != ' ' && *x != '\n')
        .collect()
}

pub fn to_statement(x: Vec<GlobalNode>) -> Vec<StatementNode> {
    x.iter()
        .map(|x| StatementNode::General(Box::new(x.clone())))
        .collect()
}

#[allow(dead_code)]
pub fn run_tests<F, T>(parser: F, test_cases: Vec<(&str, &str)>)
where
    F: Fn(&mut Parser) -> Result<T, String>,
    T: Display,
{
    for (test_case, correct_result) in test_cases {
        let lexer =
            Lexer::new(&test_case).unwrap_or_else(|_| Lexer::new("\"Lexer Error\"").unwrap());

        let mut parser_struct = Parser {
            lexer,
            semantics: Semantics::default(),
        };

        let result = parser(&mut parser_struct).unwrap().to_string();

        assert_eq!(raw_clean_string(correct_result), raw_clean_string(&result));
    }
}

pub fn verify_next_in_comma_list(
    lexer: &mut Lexer,
    end_token: TokenTypes,
    error_message: &'static str,
) -> Result<(), String> {
    if let Some(token) = lexer.peek()
        && token != end_token
    {
        let Some(future_token) = lexer.forward_peek() else {
            lexer.check(|x| x == &end_token)?;
            return Ok(());
        };

        if future_token != end_token {
            lexer.check(|x| matches!(x, TokenTypes::Operator(OperatorTypes::Comma)))?;
        } else {
            return Err(String::from(error_message));
        }
    } else if lexer.peek().is_none() {
        return Err(String::from(error_message));
    }

    return Ok(());
}

pub fn is_expression(token: &TokenTypes) -> bool {
    match token {
        TokenTypes::Literal(_) => true,
        TokenTypes::Operator(OperatorTypes::LParen) => true,
        TokenTypes::LCurlyBrace => true,
        TokenTypes::Identifier(_) => true,
        TokenTypes::Operator(op) if op.potential_unary() => true,
        TokenTypes::Keyword(KeywordTypes::Sizeof) => true,
        _ => false,
    }
}
