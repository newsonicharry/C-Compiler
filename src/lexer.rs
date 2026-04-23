use std::{
    fmt::Display,
    iter::Peekable,
    mem,
    str::{Chars, FromStr},
};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum KeywordTypes {
    Auto,
    Break,
    Case,
    Char,
    Const,
    Continue,
    Default,
    Do,
    Double,
    Else,
    Enum,
    Extern,
    Float,
    For,
    Goto,
    If,
    Int,
    Long,
    Register,
    Return,
    Short,
    Signed,
    Sizeof,
    Static,
    Struct,
    Switch,
    Typedef,
    Union,
    Unsigned,
    Void,
    Volatile,
    While,
    _Bool,
    _Complex,
    _Imaginary,
    Inline,
    Restrict,
}

impl KeywordTypes {
    const MAPPINGS: &'static [(&'static str, Self); 37] = &[
        ("auto", Self::Auto),
        ("break", Self::Break),
        ("case", Self::Case),
        ("char", Self::Char),
        ("const", Self::Const),
        ("continue", Self::Continue),
        ("default", Self::Default),
        ("do", Self::Do),
        ("double", Self::Double),
        ("else", Self::Else),
        ("enum", Self::Enum),
        ("extern", Self::Extern),
        ("float", Self::Float),
        ("for", Self::For),
        ("goto", Self::Goto),
        ("if", Self::If),
        ("int", Self::Int),
        ("long", Self::Long),
        ("register", Self::Register),
        ("return", Self::Return),
        ("short", Self::Short),
        ("signed", Self::Signed),
        ("sizeof", Self::Sizeof),
        ("static", Self::Static),
        ("struct", Self::Struct),
        ("switch", Self::Switch),
        ("typedef", Self::Typedef),
        ("union", Self::Union),
        ("unsigned", Self::Unsigned),
        ("void", Self::Void),
        ("volatile", Self::Volatile),
        ("while", Self::While),
        ("_Bool", Self::_Bool),
        ("_Complex", Self::_Complex),
        ("_Imaginary", Self::_Imaginary),
        ("inline", Self::Inline),
        ("Restrict", Self::Restrict),
    ];
}

impl FromStr for KeywordTypes {
    type Err = String;

    fn from_str(input_str: &str) -> Result<Self, Self::Err> {
        let found_value = Self::MAPPINGS
            .iter()
            .find(|(x, _)| *x == input_str)
            .map(|(_, x)| x);

        match found_value {
            Some(x) => return Ok(*x),
            None => Err("Given String is not a valid operator".to_string()),
        }
    }
}

impl Display for KeywordTypes {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let found_value = Self::MAPPINGS
            .iter()
            .find(|(_, x)| x == self)
            .map(|(x, _)| x)
            .unwrap();

        write!(display, "{found_value}")
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub enum OperatorTypes {
    #[default]
    NoOperator,
    // regular expression operators
    Divide,
    Modulus,
    BitLShift,
    BitRShift,
    Less,
    LessOrEq,
    Greater,
    GreaterOrEq,
    Equal,
    NotEqual,
    BitXOR,
    BitOr,
    And,
    Or,

    // ambiguious operators
    Inc,        // ++ can be prefix or postfix
    Dec,        // same here
    Plus,       // can be a positive number or two numbers added
    Minus,      // same
    Star,       // can be a multiply or a dereference
    Amperstand, // can be a bitwise and or a address

    // post fix operators
    LParen,
    RParen,
    LSquareBracket,
    RSquareBracket,
    DotOperator,
    ArrowOperator,

    // prefix operators
    BitNot,
    Not,

    // assignment
    SimpleAssignment,
    AddAssignment,
    SubAssignment,
    MultiAssignment,
    DivisionAssignment,
    ModulusAssignment,
    LShiftAssignment,
    RShiftAssignment,
    BitAndAssignment,
    BitXORAssignment,
    BitOrAssignment,
}

impl OperatorTypes {
    const MAPPINGS: &'static [(&'static str, Self); 40] = &[
        ("(UNKNOWN)", Self::NoOperator),
        ("*", Self::Star),
        ("/", Self::Divide),
        ("%", Self::Modulus),
        ("+", Self::Plus),
        ("-", Self::Minus),
        ("<<", Self::BitLShift),
        (">>", Self::BitRShift),
        ("<", Self::Less),
        ("<=", Self::LessOrEq),
        (">", Self::Greater),
        (">=", Self::GreaterOrEq),
        ("==", Self::Equal),
        ("!=", Self::NotEqual),
        ("&", Self::Amperstand),
        ("^", Self::BitXOR),
        ("|", Self::BitOr),
        ("&&", Self::And),
        ("||", Self::Or),
        ("(", Self::LParen),
        (")", Self::RParen),
        ("[", Self::LSquareBracket),
        ("]", Self::RSquareBracket),
        (".", Self::DotOperator),
        ("->", Self::ArrowOperator),
        ("++", Self::Inc),
        ("--", Self::Dec),
        ("~", Self::BitNot),
        ("!", Self::Not),
        ("=", Self::SimpleAssignment),
        ("+=", Self::AddAssignment),
        ("-=", Self::SubAssignment),
        ("*=", Self::MultiAssignment),
        ("/=", Self::DivisionAssignment),
        ("%=", Self::ModulusAssignment),
        ("<<=", Self::LShiftAssignment),
        (">>=", Self::RShiftAssignment),
        ("&=", Self::BitAndAssignment),
        ("^=", Self::BitXORAssignment),
        ("|=", Self::BitOrAssignment),
    ];

    fn precedence(&self) -> u8 {
        return match self {
            Self::Or => 1,
            Self::And => 2,
            Self::BitOr => 3,
            Self::BitXOR => 4,
            Self::Amperstand => 5, // BitAnd
            Self::Equal | Self::NotEqual => 6,
            Self::Greater | Self::GreaterOrEq | Self::Less | Self::LessOrEq => 7,
            Self::BitLShift | Self::BitRShift => 8,
            Self::Plus | Self::Minus => 9,
            Self::Star | Self::Divide | Self::Modulus => 10,
            _ => 0,
        };
    }
}

impl FromStr for OperatorTypes {
    type Err = String;

    fn from_str(input_str: &str) -> Result<Self, Self::Err> {
        let found_value = Self::MAPPINGS
            .iter()
            .find(|(x, _)| *x == input_str)
            .map(|(_, x)| x);

        match found_value {
            Some(x) => return Ok(*x),
            None => Err("Given String is not a valid operator".to_string()),
        }
    }
}

impl Display for OperatorTypes {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let found_value = Self::MAPPINGS
            .iter()
            .find(|(_, x)| x == self)
            .map(|(x, _)| x)
            .unwrap();

        write!(display, "{found_value}")
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum LiteralTypes {
    Float(i64, u32), // integer and shift
    Integer(u64),
    String(String),
    Character(char),
}

impl Display for LiteralTypes {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output;

        match self {
            Self::Integer(x) => output = x.to_string(),
            Self::String(x) => output = x.to_string(),
            Self::Character(x) => output = x.to_string(),

            Self::Float(base, exponent) => output = base.pow(*exponent).to_string(),
        }

        write!(display, "{output}")
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum TokenTypes {
    Identifier(String),
    Literal(LiteralTypes),
    Keyword(KeywordTypes),
    Operator(OperatorTypes),
    LCurlyBrace,
    RCurlyBrace,
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
                '{' => push_and_skip(TokenTypes::RCurlyBrace),
                '}' => push_and_skip(TokenTypes::LCurlyBrace),

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
                    lexer.tokens.push(Self::parse_operator(chars));
                }
                _ => {
                    chars.next();
                }
            }
        }

        lexer
    }

    fn parse_operator(chars: &mut Peekable<Chars<'_>>) -> TokenTypes {
        let mut final_string = String::from("");
        let mut final_operator = OperatorTypes::NoOperator;

        while let Some(&char) = chars.peek() {
            final_string += &char.to_string();

            // we can abuse that all operators build upon one another
            // meaning that all multi char operators have a previous char that is in another operator
            // this means the operator is finished when its newest version stops being valid
            let try_as_operator = OperatorTypes::from_str(&final_string);

            if try_as_operator.is_err() || char == ' ' || char == ';' {
                break;
            }

            chars.next();
            final_operator = try_as_operator.unwrap();
        }

        return TokenTypes::Operator(final_operator);
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

        for char in chars {
            if char == ' ' || char.is_ascii_punctuation() {
                break;
            }

            final_string += &char.to_string();
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
            }
        }

        write!(display, "{output}")
    }
}
