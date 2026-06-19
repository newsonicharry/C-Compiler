use std::{
    fmt::Display,
    iter::Peekable,
    str::{Chars, FromStr},
};

use crate::lexer::{
    escape_sequences::{CharType, split_string},
    language_features::{AssignmentTypes, DataTypes, KeywordTypes, LiteralTypes, OperatorTypes},
};

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub enum TokenTypes {
    #[default]
    NoToken,
    Identifier(String),
    Literal(LiteralTypes),
    Keyword(KeywordTypes),
    DataType(DataTypes),
    Operator(OperatorTypes),
    Assignment(AssignmentTypes),
    LCurlyBrace,
    RCurlyBrace,
    Comma,
    Semicolon,
}

impl Display for TokenTypes {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let final_str = &match self {
            TokenTypes::NoToken => "empty".to_string(),
            TokenTypes::LCurlyBrace => "left curly brace".to_string(),
            TokenTypes::RCurlyBrace => "right curly brace".to_string(),
            TokenTypes::Comma => "comma".to_string(),
            TokenTypes::Semicolon => "semicolon".to_string(),

            TokenTypes::Identifier(x) => format!("identifier {x}"),
            TokenTypes::Keyword(x) => format!("keyword {x}"),
            TokenTypes::DataType(x) => format!("datatype {x}"),
            TokenTypes::Operator(x) => format!("operator {x}"),
            TokenTypes::Assignment(x) => format!("assignment {x}"),
            TokenTypes::Literal(x) => format!("literal {x}"),
        };

        write!(display, "{final_str}")
    }
}

#[derive(Default, Debug)]
pub struct Lexer {
    tokens: Vec<TokenTypes>,

    set_index: usize,
    pub curr_index: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Result<Lexer, String> {
        let mut lexer = Lexer::default();
        let mut input = Self::clean_comments(input);
        input = input.chars().filter(|x| *x != '\n').collect::<String>();

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
                    lexer.tokens.push(Self::parse_string_literal(chars)?);
                }

                '\'' => {
                    lexer.tokens.push(Self::parse_char_literal(chars)?);
                }
                c if c.is_alphabetic() || c == '_' => {
                    lexer.tokens.push(Self::parse_keyword_or_identifier(chars));
                }

                c if c.is_numeric() => {
                    lexer.tokens.push(Self::parse_number_literal(chars)?);
                }
                c if c.is_ascii_punctuation() && c != ';' && c != '_' => {
                    lexer.tokens.push(Self::parse_symbol(chars));
                }
                _ => {
                    chars.next();
                }
            }
        }

        Ok(lexer)
    }

    fn clean_comments(input: &str) -> String {
        let mut final_str = String::new();

        let mut all_chars = input.chars().peekable();
        let Some(mut previous_char) = all_chars.next() else {
            return final_str;
        };

        let mut is_single_line_comment = false;
        let mut is_multi_line_comment = false;
        while let Some(char) = all_chars.next() {
            // remove comment status
            if is_single_line_comment && char == '\n' {
                is_single_line_comment = false;
                previous_char = ' '; // space because we don't want statments on different lines combining
            } else if is_multi_line_comment && (previous_char == '*' && char == '/') {
                is_multi_line_comment = false;
                previous_char = ' ';

                // we only use a continue here because this is 2 characters and thus
                // our current char which is a slash would become the next previous char
                // and that would be added
                continue;
            }

            // add comment status
            if previous_char == '/' && char == '/' {
                is_single_line_comment = true;
            } else if previous_char == '/' && char == '*' {
                is_multi_line_comment = true;
            }

            if is_single_line_comment || is_multi_line_comment {
                previous_char = char;
                continue; // skip characters that are in a comment
            }

            final_str.push(previous_char);

            let last_item = all_chars.peek().is_none();
            if last_item {
                final_str.push(char);
            }

            previous_char = char;
        }

        final_str
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

    fn parse_char_literal(chars: &mut Peekable<Chars<'_>>) -> Result<TokenTypes, String> {
        chars.next();

        let Some(mut previous_char) = chars.peek().cloned() else {
            return Err(String::from(
                "Character literal must have associated character",
            ));
        };

        chars.next();

        // we use a string here because escape seqeunces are multiple characters
        let mut final_char = String::from(previous_char);

        for char in chars.by_ref() {
            // making sure the corresponding quote we find is not just an escape sequence
            if char == '\'' && previous_char != '\\' {
                let as_char_type = final_char.parse()?;
                return Ok(TokenTypes::Literal(LiteralTypes::Character(as_char_type)));
            }

            final_char += &char.to_string();
            previous_char = char;
        }

        // means that a corresponding quote does not exist, aka not valid syntax
        Err(String::from(
            "Char literal must end with closing single quote",
        ))
    }

    fn parse_string_literal(chars: &mut Peekable<Chars<'_>>) -> Result<TokenTypes, String> {
        let mut previous_char = *chars.peek().unwrap();
        let mut final_string = String::new();

        chars.next();

        for char in chars.by_ref() {
            // making sure the corresponding quote we find is not just an escape sequence
            if char == '\"' && previous_char != '\\' {
                let as_char_list = split_string(&final_string)?;
                return Ok(TokenTypes::Literal(LiteralTypes::String(as_char_list)));
            }

            final_string += &char.to_string();
            previous_char = char;
        }

        // means that a corresponding quote does not exist, aka not valid syntax
        Err(String::from(
            "String literal must end with closing double quote",
        ))
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
        let try_as_datatype = DataTypes::from_str(&final_string);

        if let Ok(keyword) = try_as_keyword {
            return TokenTypes::Keyword(keyword);
        }

        if let Ok(data_type) = try_as_datatype {
            return TokenTypes::DataType(data_type);
        }

        return TokenTypes::Identifier(final_string);
    }

    fn parse_number_literal(chars: &mut Peekable<Chars<'_>>) -> Result<TokenTypes, String> {
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
            Err(_) => {
                return Err(String::from(&format!(
                    "Failed to convert number literal {final_string} with radix {radix} to integer"
                )));
            }
        }
    }

    pub fn check<F>(&mut self, enum_match: F) -> Result<TokenTypes, String>
    where
        F: Fn(&TokenTypes) -> bool,
    {
        let Some(token) = self.peek() else {
            return Err(String::from("Expected another token, got nothing"));
        };

        enum_match(&token)
            .then_some(self.peek().unwrap())
            .ok_or(String::from(format!(
                "Got unexpected token of type {token}"
            )))
    }

    pub fn expect<F>(&mut self, enum_match: F) -> Result<TokenTypes, String>
    where
        F: Fn(&TokenTypes) -> bool,
    {
        let result = self.check(enum_match);
        self.advance();
        result
    }

    pub fn expect_extract<F, T>(&mut self, enum_match: F) -> Result<T, String>
    where
        F: Fn(TokenTypes) -> Option<T>,
    {
        let Some(token) = self.peek() else {
            return Err(String::from("Expected another token, got nothing"));
        };

        enum_match(token)
            .and_then(|x| {
                self.advance();
                Some(x)
            })
            .ok_or(String::from(format!("Unexpected token {:?}", self.peek())))
    }

    pub fn peek(&self) -> Option<TokenTypes> {
        if self.curr_index >= self.tokens.len() {
            return None;
        }

        Some(self.tokens[self.curr_index].clone())
    }

    pub fn force_peek(&self, err_msg: &'static str) -> Result<TokenTypes, String> {
        if self.curr_index >= self.tokens.len() {
            return Err(err_msg.to_string());
        }

        Ok(self.tokens[self.curr_index].clone())
    }

    pub fn set_flag(&mut self) {
        self.set_index = self.curr_index;
    }

    pub fn recede_to_flag(&mut self) {
        self.curr_index = self.set_index;
    }

    pub fn recede(&mut self) {
        self.curr_index -= 1;
    }

    pub fn forward_peek(&self) -> Option<TokenTypes> {
        let new_index = self.curr_index + 1;

        if new_index >= self.tokens.len() {
            return None;
        }

        Some(self.tokens[new_index].clone())
    }

    pub fn advance(&mut self) {
        self.curr_index += 1;
    }

    pub fn next(&mut self) -> Option<TokenTypes> {
        self.peek().and_then(|token_type| {
            self.advance();
            Some(token_type)
        })
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
                TokenTypes::DataType(x) => add_token("DATATYPE", &x.to_string()),

                TokenTypes::Literal(literal_type) => match literal_type {
                    LiteralTypes::String(x) => add_token("STRING", &x.to_string()),
                    LiteralTypes::Integer(x) => add_token("INTEGER", &x.to_string()),
                    LiteralTypes::Character(x) => add_token("CHAR", &x.to_string()),
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
