use std::{fmt::Display, str::FromStr};

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

impl_from_str_for_enum!(KeywordTypes);
impl_display_for_enum!(KeywordTypes);

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

    // prefix operators
    BitNot,
    Not,
}

impl OperatorTypes {
    const MAPPINGS: &'static [(&'static str, Self); 29] = &[
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
    ];

    pub fn precedence(&self) -> u8 {
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

impl_from_str_for_enum!(OperatorTypes);
impl_display_for_enum!(OperatorTypes);

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
