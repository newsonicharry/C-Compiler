use std::{
    fmt::Display,
    iter::Peekable,
    str::{CharIndices, FromStr},
};

use crate::lexer::{
    escape_sequences::split_string,
    language_features::{AssignmentTypes, DataTypes, KeywordTypes, LiteralTypes, OperatorTypes},
    number_parser::parse_number_literal,
};

#[derive(Default, Clone, PartialEq, Debug)]
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
    Semicolon,
}

impl Display for TokenTypes {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let final_str = &match self {
            TokenTypes::NoToken => "empty".to_string(),
            TokenTypes::LCurlyBrace => "left curly brace".to_string(),
            TokenTypes::RCurlyBrace => "right curly brace".to_string(),
            TokenTypes::Semicolon => "semicolon".to_string(),

            TokenTypes::Identifier(x) => format!("identifier {x}"),
            TokenTypes::Keyword(x) => format!("keyword {x}"),
            TokenTypes::DataType(x) => format!("datatype {x}"),
            TokenTypes::Assignment(x) => format!("assignment {x}"),
            TokenTypes::Literal(x) => format!("literal {x}"),
            TokenTypes::Operator(x) => format!("operator {}", x.operator_in_words()),
        };

        write!(display, "{final_str}")
    }
}

#[derive(Default, Debug, Clone)]
pub struct Lexer {
    tokens: Vec<(TokenTypes, usize)>,
    set_index: usize,
    curr_index: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Result<Lexer, String> {
        let mut lexer = Lexer::default();

        let chars = &mut input.char_indices().peekable();
        while let Some((index, char)) = chars.peek().cloned() {
            let next_char = chars.clone().nth(1);
            let next_char_is_digit = next_char.unwrap_or((0, ' ')).1.is_digit(10);

            if next_char.is_some() && char == '/' && next_char.unwrap().1 == '/' {
                Self::clean_single_line_comment(chars);
                continue;
            }

            if next_char.is_some() && char == '/' && next_char.unwrap().1 == '*' {
                Self::clean_multi_line_comment(chars);
                continue;
            }

            match char {
                ';' => {
                    lexer.tokens.push((TokenTypes::Semicolon, index));
                    chars.next();
                }
                '{' => {
                    lexer.tokens.push((TokenTypes::LCurlyBrace, index));
                    chars.next();
                }
                '}' => {
                    lexer.tokens.push((TokenTypes::RCurlyBrace, index));
                    chars.next();
                }

                '\"' => {
                    lexer
                        .tokens
                        .push((Self::parse_string_literal(chars)?, index));
                }

                '\'' => {
                    lexer.tokens.push((Self::parse_char_literal(chars)?, index));
                }
                c if c.is_alphabetic() || c == '_' => {
                    lexer
                        .tokens
                        .push((Self::parse_keyword_or_identifier(chars), index));
                }

                c if c.is_digit(10) || (c == '.' && next_char_is_digit) => {
                    lexer.tokens.push((parse_number_literal(chars)?, index));
                }
                c if c.is_ascii_punctuation() && c != ';' && c != '_' => {
                    lexer.tokens.push((Self::parse_symbol(chars), index));
                }
                _ => {
                    chars.next();
                }
            }
        }

        Ok(lexer)
    }

    fn clean_single_line_comment(chars: &mut Peekable<CharIndices<'_>>) {
        while let Some((_, char)) = chars.next() {
            if char == '\n' {
                break;
            }
        }
    }

    fn clean_multi_line_comment(chars: &mut Peekable<CharIndices<'_>>) {
        let mut previous_char = chars.next().unwrap().1;
        while let Some((_, char)) = chars.next() {
            if previous_char == '*' && char == '/' {
                break;
            }
            previous_char = char;
        }
    }

    fn parse_ellipsis(chars: &mut Peekable<CharIndices<'_>>) -> Option<TokenTypes> {
        let mut chars_copy = chars.clone();

        let mut i = 0;
        while let Some((_, curr_char)) = chars_copy.next()
            && i < 3
        {
            if curr_char != '.' {
                return None;
            }

            i += 1;
        }

        if i < 3 {
            return None;
        }

        chars.next();
        chars.next();
        chars.next();

        return Some(TokenTypes::Operator(OperatorTypes::Ellipsis));
    }

    fn parse_symbol(chars: &mut Peekable<CharIndices<'_>>) -> TokenTypes {
        if let Some(ellipsis) = Self::parse_ellipsis(chars) {
            return ellipsis;
        }

        let mut final_string = String::from("");
        let mut final_type = TokenTypes::NoToken;

        while let Some(&(_, char)) = chars.peek() {
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

    fn parse_char_literal(chars: &mut Peekable<CharIndices<'_>>) -> Result<TokenTypes, String> {
        chars.next();

        let Some((_, mut previous_char)) = chars.peek().cloned() else {
            return Err(String::from(
                "Character literal must have associated character",
            ));
        };

        chars.next();

        // we use a string here because escape seqeunces are multiple characters
        let mut final_char = String::from(previous_char);

        for (_, char) in chars.by_ref() {
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

    fn parse_string_literal(chars: &mut Peekable<CharIndices<'_>>) -> Result<TokenTypes, String> {
        let mut previous_char = chars.peek().unwrap().1;
        let mut final_string = String::new();

        chars.next();

        for (_, char) in chars.by_ref() {
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

    fn parse_keyword_or_identifier(chars: &mut Peekable<CharIndices<'_>>) -> TokenTypes {
        let mut final_string = String::from("");

        while let Some((_, char)) = chars.peek().cloned() {
            // since a function or variable can have a underscore we cant end on that
            if char == ' ' || char == '\n' || char.is_ascii_punctuation() && char != '_' {
                break;
            }
            final_string += &chars.next().unwrap().1.to_string();
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

    fn get_all_tokens_types() -> Vec<TokenTypes> {
        let all_operators = OperatorTypes::MAPPINGS.map(|x| TokenTypes::Operator(x.1));
        let all_assignment_types = AssignmentTypes::MAPPINGS.map(|x| TokenTypes::Assignment(x.1));
        let all_data_types = DataTypes::MAPPINGS.map(|x| TokenTypes::DataType(x.1));
        let all_keyword_types = KeywordTypes::MAPPINGS.map(|x| TokenTypes::Keyword(x.1));

        let mut all_token_types = vec![
            TokenTypes::LCurlyBrace,
            TokenTypes::RCurlyBrace,
            TokenTypes::Semicolon,
            TokenTypes::Identifier(String::new()),
        ];
        all_token_types.extend(all_operators);
        all_token_types.extend(all_assignment_types);
        all_token_types.extend(all_data_types);
        all_token_types.extend(all_keyword_types);

        all_token_types
    }

    fn brute_force_expect_correct_token<F>(enum_match: F) -> Option<TokenTypes>
    where
        F: Fn(&TokenTypes) -> bool,
    {
        for token_type in Self::get_all_tokens_types() {
            if enum_match(&token_type) {
                return Some(token_type);
            }
        }

        None
    }

    fn brute_force_expect_extract_correct_token<F, T>(enum_match: F) -> Option<TokenTypes>
    where
        F: Fn(TokenTypes) -> Option<T>,
    {
        for token_type in Self::get_all_tokens_types() {
            if enum_match(token_type.clone()).is_some() {
                return Some(token_type);
            }
        }

        None
    }

    pub fn check<F>(&mut self, enum_match: F) -> Result<TokenTypes, String>
    where
        F: Fn(&TokenTypes) -> bool,
    {
        let Some(token) = self.peek() else {
            if let Some(correct_token) = Self::brute_force_expect_correct_token(enum_match) {
                return Err(format!(
                    "Expected another token of type {}, got nothing",
                    correct_token.to_string().to_lowercase()
                ));
            }

            return Err(String::from("Expected another token, got nothing"));
        };

        if !enum_match(&token) {
            if let Some(correct_token) = Self::brute_force_expect_correct_token(enum_match) {
                return Err(format!(
                    "Got unexpected token of type {token}, expected token of type {}",
                    correct_token.to_string().to_lowercase()
                ));
            }

            return Err(format!("Got unexpected token of type {token}"));
        }

        Ok(self.peek().unwrap())
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
            if let Some(correct_token) = Self::brute_force_expect_extract_correct_token(enum_match)
            {
                return Err(format!(
                    "Expected another token of type {}, got nothing",
                    correct_token.to_string().to_lowercase()
                ));
            }

            return Err(String::from("Expected another token, got nothing"));
        };

        let result = enum_match(token.clone());

        if result.is_none() {
            if let Some(correct_token) = Self::brute_force_expect_extract_correct_token(enum_match)
            {
                return Err(format!(
                    "Got unexpected token of type {token}, expected token of type {}",
                    correct_token.to_string().to_lowercase()
                ));
            }

            return Err(String::from(format!(
                "Got unexpected token {}",
                self.peek().unwrap()
            )));
        }

        self.advance();
        Ok(result.unwrap())
    }

    pub fn peek(&self) -> Option<TokenTypes> {
        if self.curr_index >= self.tokens.len() {
            return None;
        }

        Some(self.tokens[self.curr_index].0.clone())
    }

    pub fn force_peek(&self, err_msg: &'static str) -> Result<TokenTypes, String> {
        if self.curr_index >= self.tokens.len() {
            return Err(err_msg.to_string());
        }

        Ok(self.tokens[self.curr_index].0.clone())
    }

    pub fn set_flag(&mut self) {
        self.set_index = self.curr_index;
    }

    pub fn recede_to_flag(&mut self) {
        self.curr_index = self.set_index;
    }

    pub fn forward_peek(&self) -> Option<TokenTypes> {
        let new_index = self.curr_index + 1;

        if new_index >= self.tokens.len() {
            return None;
        }

        Some(self.tokens[new_index].0.clone())
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

    pub fn get_tokens(&self) -> &Vec<(TokenTypes, usize)> {
        &self.tokens
    }

    pub fn last_index(&self) -> usize {
        let curr_index = self.curr_index.min(self.tokens.len().saturating_sub(2));

        self.tokens[curr_index].1
    }
}

impl Display for Lexer {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();

        let mut add_token = |name: &str, value: &str| {
            output.push_str(&(String::from("[") + name + ": " + value + " ]\n"))
        };

        for (token, _) in self.get_tokens() {
            match token {
                TokenTypes::Identifier(x) => add_token("IDENTIFIER", x),
                TokenTypes::Operator(x) => add_token("OPERATOR", &x.to_string()),
                TokenTypes::Assignment(x) => add_token("ASSIGNMENT", &x.to_string()),
                TokenTypes::Keyword(x) => add_token("KEYWORD", &x.to_string()),
                TokenTypes::DataType(x) => add_token("DATATYPE", &x.to_string()),

                TokenTypes::Literal(literal_type) => match literal_type {
                    LiteralTypes::String(x) => add_token("STRING", &x.to_string()),
                    LiteralTypes::Integer(x) => add_token("INTEGER", &x.to_string()),
                    LiteralTypes::Float(x) => add_token("FLOAT", &x.to_string()),
                    LiteralTypes::Character(x) => add_token("CHAR", &x.to_string()),
                },

                TokenTypes::Semicolon => add_token("SEMICOLON", ";"),
                TokenTypes::LCurlyBrace => add_token("LCURLYBRACE", "{"),
                TokenTypes::RCurlyBrace => add_token("RCURLYBRACE", "}"),
                TokenTypes::NoToken => add_token("(NO TOKEN)", "<WARNING NO TOKEN> "),
            }
        }

        write!(display, "{output}")
    }
}
