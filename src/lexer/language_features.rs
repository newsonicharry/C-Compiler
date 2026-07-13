use std::{fmt::Display, str::FromStr};

use crate::lexer::{
    escape_sequences::{CharList, CharType},
    number_parser::{FloatType, IntType},
};

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
    For,
    Goto,
    If,
    Return,
    Sizeof,
    Struct,
    Switch,
    Union,
    While,
}

impl KeywordTypes {
    pub const MAPPINGS: &'static [(&'static str, Self); 16] = &[
        ("break", Self::Break),
        ("case", Self::Case),
        ("continue", Self::Continue),
        ("default", Self::Default),
        ("do", Self::Do),
        ("else", Self::Else),
        ("enum", Self::Enum),
        ("for", Self::For),
        ("goto", Self::Goto),
        ("if", Self::If),
        ("return", Self::Return),
        ("sizeof", Self::Sizeof),
        ("struct", Self::Struct),
        ("switch", Self::Switch),
        ("union", Self::Union),
        ("while", Self::While),
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
    Extern,
    Float,
    Inline,
    Int,
    Long,
    Register,
    Restrict,
    Short,
    Signed,
    Static,
    Typedef,
    Unsigned,
    Void,
    Volatile,
    _Bool,
    _Complex,
}

impl DataTypes {
    pub const MAPPINGS: &'static [(&'static str, Self); 21] = &[
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
        ("_Complex", Self::_Complex),
        ("register", Self::Register),
        ("static", Self::Static),
        ("extern", Self::Extern),
        ("inline", Self::Inline),
        ("typedef", Self::Typedef),
    ];

    pub fn is_storage_specifier(&self) -> bool {
        match self {
            Self::Register | Self::Static | Self::Extern | Self::Auto | Self::Typedef => true,
            _ => false,
        }
    }

    pub fn is_qualifier(&self) -> bool {
        match *self {
            //  auto is not technically a qualifier but we assume it is here
            Self::Const | Self::Volatile | Self::Restrict | Self::Auto => true,
            _ => false,
        }
    }

    pub fn is_modifier(&self) -> bool {
        match *self {
            // _Complex is techinally its own distict type but doing that would be a headache so this is used instead
            Self::Signed | Self::Unsigned | Self::Short | Self::Long | Self::_Complex => true,
            _ => false,
        }
    }

    pub fn is_function_specifier(&self) -> bool {
        match *self {
            Self::Inline => true,
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
    pub const MAPPINGS: &'static [(&'static str, Self); 11] = &[
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

    Comma,
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
    QuestionMark,

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
    pub const MAPPINGS: &'static [(&'static str, Self); 32] = &[
        ("(UNKNOWN)", Self::NoOperator),
        (",", Self::Comma),
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
        ("?", Self::QuestionMark),
    ];

    pub fn operator_in_words(&self) -> String {
        let output = match self {
            Self::NoOperator => "(UNKNOWN)",
            Self::Comma => "Comma",
            Self::Star => "Star",
            Self::Divide => "Slash",
            Self::Modulus => "Modulus",
            Self::Plus => "Plus",
            Self::Minus => "Minus",
            Self::BitLShift => "Bitwise Left Shift",
            Self::BitRShift => "Bitwise Right Shift",
            Self::Less => "Less than",
            Self::LessOrEq => "Less than or Equal",
            Self::Greater => "Greater than",
            Self::GreaterOrEq => "Greater than or Equal",
            Self::Equal => "Equality",
            Self::NotEqual => "Inequality",
            Self::Amperstand => "Amperstand",
            Self::BitXOR => "Bit XOR",
            Self::BitOr => "Bit Or",
            Self::And => "Bit And",
            Self::Or => "Or",
            Self::LParen => "Left Parenthesis",
            Self::RParen => "Right Parenthesis",
            Self::LSquareBracket => "Left Square Bracket",
            Self::RSquareBracket => "Right Square Bracket",
            Self::DotOperator => "Dot",
            Self::ArrowOperator => "Arrow",
            Self::Inc => "Increment",
            Self::Dec => "Decrement",
            Self::BitNot => "Bit Not",
            Self::Not => "Not",
            Self::Colon => "Colon",
            Self::QuestionMark => "Question Mark",
        };

        String::from(output)
    }

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
}

impl_from_str_for_enum!(OperatorTypes);
impl_display_for_enum!(OperatorTypes);

#[derive(Clone, PartialEq, Debug)]
pub enum LiteralTypes {
    Float(FloatType),
    Integer(IntType),
    String(CharList),
    Character(CharType),
}

impl Display for LiteralTypes {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output;

        match self {
            Self::String(x) => output = x.to_string(),
            Self::Character(x) => output = x.to_string(),

            Self::Integer(x) => output = x.to_string(),
            Self::Float(x) => output = x.to_string(),
        }

        write!(display, "{output}")
    }
}
