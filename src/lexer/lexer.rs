use std::{
    fmt::Display,
    iter::Peekable,
    mem,
    str::{Chars, FromStr},
};

use crate::lexer::language_features::{AssignmentTypes, KeywordTypes, LiteralTypes, OperatorTypes};

#[derive(Default, Clone, PartialEq, Eq)]
pub enum TokenTypes {
    #[default]
    NoToken,
    Identifier(String),
    Literal(LiteralTypes),
    Keyword(KeywordTypes),
    Operator(OperatorTypes),
    Assignment(AssignmentTypes),
    LCurlyBrace,
    RCurlyBrace,
    Comma,
    Semicolon,
}

#[derive(Default)]
pub struct Lexer {
    tokens: Vec<TokenTypes>,

    pub curr_index: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Lexer {
        let mut lexer = Lexer::default();

        let chars = &mut input.chars().peekable();
        loop {
            if let None = chars.peek() {
                break;
            }

            let char = *chars.peek().unwrap();

            let mut push_and_skip = |token: TokenTypes| {
                lexer.tokens.push(token);
                chars.next();
            };

            match char {
                ';' => push_and_skip(TokenTypes::Semicolon),
                ',' => push_and_skip(TokenTypes::Comma),
                '{' => push_and_skip(TokenTypes::LCurlyBrace),
                '}' => push_and_skip(TokenTypes::RCurlyBrace),

                '\"' => {
                    lexer
                        .tokens
                        .push(Self::parse_string_literal(chars).unwrap());
                }
                c if c.is_alphabetic() || c == '_' => {
                    lexer.tokens.push(Self::parse_keyword_or_identifier(chars));
                }

                c if c.is_numeric() => {
                    lexer
                        .tokens
                        .push(Self::parse_number_literal(chars).unwrap());
                }
                c if c.is_ascii_punctuation() && c != ';' && c != '_' => {
                    lexer.tokens.push(Self::parse_symbol(chars));
                }
                _ => {
                    chars.next();
                }
            }
        }

        lexer
    }

    fn parse_symbol(chars: &mut Peekable<Chars<'_>>) -> TokenTypes {
        let mut final_string = String::from("");
        let mut final_type = TokenTypes::NoToken;

        while let Some(&char) = chars.peek() {
            final_string += &char.to_string();

            // we can abuse that all operators build upon one another
            // meaning that all multi char operators have a previous char that is in another operator
            // this means the operator is finished when its newest version stops being valid
            let try_as_operator = OperatorTypes::from_str(&final_string);
            let try_as_assignment = AssignmentTypes::from_str(&final_string);

            if (try_as_operator.is_err() && try_as_assignment.is_err())
                || char == ' '
                || char == ';'
            {
                break;
            }

            chars.next();

            if try_as_operator.is_ok() {
                final_type = TokenTypes::Operator(try_as_operator.unwrap());
            } else {
                final_type = TokenTypes::Assignment(try_as_assignment.unwrap());
            }
        }

        return final_type;
    }

    fn parse_string_literal(chars: &mut Peekable<Chars<'_>>) -> Result<TokenTypes, ()> {
        let mut previous_char = *chars.peek().unwrap();
        let mut final_string = String::from("\"");

        chars.next();

        for char in chars.by_ref() {
            // making sure the corresponding quote we find is not just an escape sequence
            if char == '\"' && previous_char != '\\' {
                return Ok(TokenTypes::Literal(LiteralTypes::String(
                    final_string + "\"",
                )));
            }

            final_string += &char.to_string();
            previous_char = char;
        }

        // means that a corresponding quote does not exist, aka not valid syntax
        Err(())
    }

    fn parse_keyword_or_identifier(chars: &mut Peekable<Chars<'_>>) -> TokenTypes {
        let mut final_string = String::from("");

        while let Some(&char) = chars.peek() {
            // since a function or variable can have a underscore we cant end on that
            if char == ' ' || char.is_ascii_punctuation() && char != '_' {
                break;
            }
            final_string += &chars.next().unwrap().to_string();
        }

        let try_as_keyword = KeywordTypes::from_str(&final_string);

        if let Ok(keyword) = try_as_keyword {
            return TokenTypes::Keyword(keyword);
        }

        return TokenTypes::Identifier(final_string);
    }

    fn parse_number_literal(chars: &mut Peekable<Chars<'_>>) -> Result<TokenTypes, ()> {
        let mut final_string = String::from("");

        while let Some(&char) = chars.peek() {
            if char == ' ' || char.is_ascii_punctuation() {
                break;
            }

            final_string += &chars.next().unwrap().to_string();
        }

        enum PrefixTypes {
            Decimal,
            Hexadecimal,
            Octal,
            Binary,
        }

        let mut prefix_type = PrefixTypes::Decimal;

        if final_string.starts_with('0') {
            if let Some(next_char) = final_string.chars().nth(1) {
                match next_char {
                    'b' | 'B' => prefix_type = PrefixTypes::Binary,
                    'x' | 'X' => prefix_type = PrefixTypes::Hexadecimal,
                    _ => prefix_type = PrefixTypes::Octal,
                }
            }
        }

        let radix: u32;

        match prefix_type {
            PrefixTypes::Hexadecimal => {
                final_string = final_string
                    .to_lowercase()
                    .trim_start_matches("0x")
                    .to_string();
                radix = 16;
            }

            PrefixTypes::Binary => {
                final_string = final_string
                    .to_lowercase()
                    .trim_start_matches("0b")
                    .to_string();
                radix = 2;
            }

            PrefixTypes::Octal => {
                final_string.remove(0);
                radix = 8;
            }

            PrefixTypes::Decimal => radix = 10,
        }

        match u64::from_str_radix(&final_string.to_string(), radix) {
            Ok(num) => return Ok(TokenTypes::Literal(LiteralTypes::Integer(num))),
            Err(_) => return Err(()),
        }
    }

    pub fn is_same_variant(&self, token: &TokenTypes) -> bool {
        let curr_token = self.peek();

        if let Some(unwrapped_token) = curr_token {
            return mem::discriminant(&unwrapped_token) == mem::discriminant(token);
        }

        return false;
    }

    pub fn expect(&mut self, token: TokenTypes) -> Result<(), ()> {
        if self.is_same_variant(&token) {
            self.advance();
            return Ok(());
        }

        Err(())
    }

    pub fn peek(&self) -> Option<TokenTypes> {
        if self.curr_index >= self.tokens.len() {
            return None;
        }

        Some(self.tokens[self.curr_index].clone())
    }

    pub fn cant_peek(&self) -> bool {
        self.curr_index < self.tokens.len()
    }

    pub fn next(&mut self) -> Option<TokenTypes> {
        let next_token = self.peek();

        if next_token.is_some() {
            self.advance();
        }

        return next_token;
    }

    pub fn advance(&mut self) {
        self.curr_index += 1;
    }

    pub fn get_tokens(&self) -> &Vec<TokenTypes> {
        &self.tokens
    }
}

impl Display for Lexer {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();

        let mut add_token = |name: &str, value: &str| {
            output.push_str(&(String::from("[") + name + ": " + value + "]\n"))
        };

        for token in self.get_tokens() {
            match token {
                TokenTypes::Identifier(x) => add_token("IDENTIFIER", x),
                TokenTypes::Operator(x) => add_token("OPERATOR", &x.to_string()),
                TokenTypes::Assignment(x) => add_token("ASSIGNMENT", &x.to_string()),
                TokenTypes::Keyword(x) => add_token("KEYWORD", &x.to_string()),

                TokenTypes::Literal(literal_type) => match literal_type {
                    LiteralTypes::String(x) => add_token("STRING", &x.to_string()),
                    LiteralTypes::Integer(x) => add_token("INTEGER", &x.to_string()),
                    _ => {
                        todo!()
                    }
                },

                TokenTypes::Semicolon => add_token("SEMICOLON", ";"),
                TokenTypes::LCurlyBrace => add_token("LCURLYBRACE", "{"),
                TokenTypes::RCurlyBrace => add_token("RCURLYBRACE", "}"),
                TokenTypes::Comma => add_token("COMMA", ","),
                TokenTypes::NoToken => add_token("(NO TOKEN)", "<WARNING NO TOKEN> "),
            }
        }

        write!(display, "{output}")
    }
}
