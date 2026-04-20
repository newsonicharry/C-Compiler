use std::fmt::Display;

#[derive(Clone, PartialEq, Eq)]
pub enum KeywordTypes {
    Int,
    For,
    If,
    While,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
pub enum OperatorTypes {
    #[default]
    NoOperator,
    Plus,
    Minus,
    Star,
    Slash,
}

impl Display for OperatorTypes {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output;

        match self {
            OperatorTypes::NoOperator => output = "(NONE)",
            OperatorTypes::Plus => output = "+",
            OperatorTypes::Minus => output = "-",
            OperatorTypes::Star => output = "*",
            OperatorTypes::Slash => output = "/",
        }

        write!(display, "{output}")
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum TokenTypes {
    Identifier(String),
    Number(i64),
    Keyword(KeywordTypes),
    Operator(OperatorTypes),
    LParen,
    RParen,
    LBRace,
    RBrace,
    Comma,
    Semicolon,
}

// impl TokenTypes {}

#[derive(Default)]
pub struct Lexer {
    tokens: Vec<TokenTypes>,
    curr_index: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Lexer {
        let mut lexer = Lexer::default();

        for char in input.chars() {
            if char.is_numeric() {
                lexer
                    .tokens
                    .push(TokenTypes::Number(char.to_digit(10).unwrap().into()));
            } else {
                lexer.tokens.push(TokenTypes::Operator(OperatorTypes::Plus));
            }
        }

        lexer
    }

    pub fn peek(&self) -> Option<TokenTypes> {
        if self.curr_index >= self.tokens.len() {
            return None;
        }

        Some(self.tokens[self.curr_index].clone())
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
                TokenTypes::Number(x) => add_token("NUMBER", &x.to_string()),
                TokenTypes::Operator(x) => add_token("OPERATOR", &x.to_string()),

                _ => {}
            }
        }

        write!(display, "{output}")
    }
}
