use crate::lexer::lexer::{Lexer, TokenTypes};
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

#[allow(dead_code)]
pub fn run_tests<F, T>(parser: F, test_cases: Vec<(&str, &str)>)
where
    F: Fn(&mut Lexer) -> Result<T, String>,
    T: Display,
{
    for (test_case, correct_result) in test_cases {
        let mut lexer =
            Lexer::new(&test_case).unwrap_or_else(|_| Lexer::new("\"Lexer Error\"").unwrap());

        let result = parser(&mut lexer).unwrap().to_string();

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
            return Err(String::from(error_message));
        };

        if future_token != end_token {
            lexer.expect(|x| matches!(x, TokenTypes::Comma))?;
        }
    } else if lexer.peek().is_none() {
        return Err(String::from(error_message));
    }

    return Ok(());
}
