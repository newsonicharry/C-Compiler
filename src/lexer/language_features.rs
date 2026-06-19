use std::{fmt::Display, str::FromStr};

use crate::lexer::escape_sequences::{CharList, CharType};

macro_rules! impl_from_str_for_enum {
    ($name:ty) => {
        impl FromStr for $name {
            type Err = String;

            fn from_str(input_str: &str) -> Result<Self, Self::Err> {
                let found_value = Self::MAPPINGS
                    .iter()
                    .find(|(x, _)| *x == input_str)
                    .map(|(_, x)| x);

                match found_value {
                    Some(x) => return Ok(*x),
                    None => Err("Given String does not appear in the mappings".to_string()),
                }
            }
        }
    };
}

macro_rules! impl_display_for_enum {
    ($name:ty) => {
        impl Display for $name {
            fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let found_value = Self::MAPPINGS
                    .iter()
                    .find(|(_, x)| x == self)
                    .map(|(x, _)| x)
                    .unwrap();

                write!(display, "{found_value}")
            }
        }
    };
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum KeywordTypes {
    Break,
    Case,
    Continue,
    Default,
    Do,
    Else,
    Enum,
    Extern,
    For,
    Goto,
    If,
    Register,
    Return,
    Sizeof,
    Static,
    Struct,
    Switch,
    Typedef,
    Union,
    While,
    _Complex,
    _Imaginary,
    Inline,
}

impl KeywordTypes {
    const MAPPINGS: &'static [(&'static str, Self); 23] = &[
        ("break", Self::Break),
        ("case", Self::Case),
        ("continue", Self::Continue),
        ("default", Self::Default),
        ("do", Self::Do),
        ("else", Self::Else),
        ("enum", Self::Enum),
        ("extern", Self::Extern),
        ("for", Self::For),
        ("goto", Self::Goto),
        ("if", Self::If),
        ("register", Self::Register),
        ("return", Self::Return),
        ("sizeof", Self::Sizeof),
        ("static", Self::Static),
        ("struct", Self::Struct),
        ("switch", Self::Switch),
        ("typedef", Self::Typedef),
        ("union", Self::Union),
        ("while", Self::While),
        ("_Complex", Self::_Complex),
        ("_Imaginary", Self::_Imaginary),
        ("inline", Self::Inline),
    ];
}

impl_from_str_for_enum!(KeywordTypes);
impl_display_for_enum!(KeywordTypes);

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub enum DataTypes {
    #[default]
    NoType,
    Auto,
    Char,
    Const,
    Double,
    Float,
    Int,
    Long,
    Short,
    Signed,
    Unsigned,
    Restrict,
    Void,
    Volatile,
    _Bool,
}

impl DataTypes {
    const MAPPINGS: &'static [(&'static str, Self); 15] = &[
        ("NOTYPE", Self::NoType),
        ("char", Self::Char),
        ("auto", Self::Auto),
        ("const", Self::Const),
        ("double", Self::Double),
        ("float", Self::Float),
        ("int", Self::Int),
        ("long", Self::Long),
        ("short", Self::Short),
        ("signed", Self::Signed),
        ("unsigned", Self::Unsigned),
        ("restrict", Self::Restrict),
        ("void", Self::Void),
        ("volatile", Self::Volatile),
        ("_Bool", Self::_Bool),
    ];

    pub fn is_qualifier(&self) -> bool {
        match *self {
            // auto is not technically a qualifier but we assume it is here
            Self::Const | Self::Volatile | Self::Restrict | Self::Auto => true,
            _ => false,
        }
    }

    pub fn is_modifier(&self) -> bool {
        match *self {
            Self::Signed | Self::Unsigned | Self::Short | Self::Long => true,
            _ => false,
        }
    }
}

impl_from_str_for_enum!(DataTypes);
impl_display_for_enum!(DataTypes);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AssignmentTypes {
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

impl AssignmentTypes {
    const MAPPINGS: &'static [(&'static str, Self); 11] = &[
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
}

impl_from_str_for_enum!(AssignmentTypes);
impl_display_for_enum!(AssignmentTypes);

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

    // Bitfields, ternary, goto, switch
    Colon,

    // ambiguious operators
    Inc,        // ++ can be prefix or postfix
    Dec,        // -- can be prefix or postfix
    Plus,       // can be a positive number or two numbers added
    Minus,      // can be a negative number or two numbers subtracted
    Star,       // can be a multiply or a dereference
    Amperstand, // can be a bitwise and or a address

    // post fix operators
    LParen,
    RParen,
    LSquareBracket,
    RSquareBracket,
    DotOperator,
    ArrowOperator,

    // unary operators
    BitNot,
    Not,
}

impl OperatorTypes {
    const MAPPINGS: &'static [(&'static str, Self); 30] = &[
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
        (":", Self::Colon),
    ];

    pub fn potential_unary(&self) -> bool {
        match self {
            Self::Inc
            | Self::Dec
            | Self::Plus
            | Self::Minus
            | Self::Star
            | Self::Amperstand
            | Self::BitNot
            | Self::Not => true,

            _ => false,
        }
    }

    pub fn potential_postfix(&self) -> bool {
        match self {
            Self::LParen
            | Self::RParen
            | Self::LSquareBracket
            | Self::RSquareBracket
            | Self::DotOperator
            | Self::ArrowOperator
            | Self::Inc
            | Self::Dec => return true,
            _ => return false,
        }
    }

    pub fn precedence(&self) -> u8 {
        match self {
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
        }
    }
}

impl_from_str_for_enum!(OperatorTypes);
impl_display_for_enum!(OperatorTypes);

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum LiteralTypes {
    Float(i64, u32), // integer and shift
    Integer(u64),
    String(CharList),
    Character(CharType),
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
