use std::fmt::Display;

use crate::{
    lexer::{language_features::DataTypes, lexer::TokenTypes},
    parser::parser::Parser,
};

#[derive(Clone, Copy, Debug, PartialEq, Default, PartialOrd, Ord, Eq)]
pub enum BaseType {
    #[default]
    Short,
    Long,

    Int,
    _Bool,
    Char,
    Float,
    Double,
    LongDouble,
    Void,
}

impl BaseType {
    pub fn is_floating(&self) -> bool {
        match self {
            Self::Float | Self::Double | Self::LongDouble => true,
            _ => false,
        }
    }

    pub fn num_bits(&self) -> u8 {
        match self {
            Self::Short => 16,
            Self::Long => 64,
            Self::Int => 32,
            Self::_Bool => 1,
            Self::Char => 8,
            Self::Float => 32,
            Self::Double => 64,
            Self::LongDouble => 128,
            Self::Void => 0,
        }
    }
}

impl Display for BaseType {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Self::Int => "int",
            Self::_Bool => "_Bool",
            Self::Char => "char",
            Self::Short => "short",
            Self::Long => "long",
            Self::Float => "float",
            Self::Double => "double",
            Self::LongDouble => "long double",
            Self::Void => "void",
        };

        write!(display, "{output}")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default, PartialOrd, Ord, Eq)]
pub enum Signedness {
    #[default]
    NoSign,
    Signed,
    Unsigned,
}

impl Display for Signedness {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Self::Signed => "signed",
            Self::Unsigned => "unsigned",
            Self::NoSign => "nosign",
        };

        write!(display, "{output}")
    }
}

impl Signedness {
    pub fn has_sign(&self) -> bool {
        !matches!(self, Signedness::NoSign)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Storage {
    #[default]
    Auto,
    Register,
    Static,
    Extern,
}

impl Display for Storage {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Self::Auto => "auto",
            Self::Register => "register",
            Self::Static => "static",
            Self::Extern => "extern",
        };

        write!(display, "{output}")
    }
}

#[derive(Clone, Debug, PartialEq, Default, PartialOrd, Ord, Eq)]
pub enum Qualifiers {
    #[default]
    Const,
    Volatile,
    Restrict,
}

impl Display for Qualifiers {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            Self::Const => "const",
            Self::Volatile => "volatile",
            Self::Restrict => "restrict",
        };

        write!(display, "{output}")
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SimpleType {
    pub base_type: Option<BaseType>,
    pub storage: Option<Storage>,
    pub signedness: Signedness,
    pub qualifiers: Vec<Qualifiers>,
    pub is_inline_function: bool,
    pub is_typedef: bool,
}

impl Display for SimpleType {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();

        match self.is_inline_function {
            true => output.push_str(&format!("inline ")),
            false if self.storage.is_some() => {
                output.push_str(&format!("{} ", self.storage.unwrap()))
            }

            _ => {}
        };

        for qualifier in &self.qualifiers {
            output.push_str(&format!("{qualifier} "));
        }

        if matches!(self.signedness, Signedness::Unsigned) {
            output.push_str("unsigned ");
        }

        if let Some(base_type) = self.base_type {
            output.push_str(&base_type.to_string());
        }

        write!(display, "{output}")
    }
}

static TYPE_LOOKUP: [(&'static [BaseType], &'static BaseType); 13] = [
    (&[BaseType::_Bool], &BaseType::_Bool),
    (&[BaseType::Char], &BaseType::Char),
    (&[BaseType::Short], &BaseType::Short),
    (&[BaseType::Int], &BaseType::Int),
    (&[BaseType::Long], &BaseType::Long),
    (&[BaseType::Double], &BaseType::Double),
    (&[BaseType::Void], &BaseType::Void),
    (&[BaseType::Long, BaseType::Long], &BaseType::Long),
    (&[BaseType::Long, BaseType::Double], &BaseType::LongDouble),
    (&[BaseType::Float], &BaseType::Float),
    (&[BaseType::Short, BaseType::Int], &BaseType::Short),
    (&[BaseType::Long, BaseType::Int], &BaseType::Long),
    (
        &[BaseType::Long, BaseType::Long, BaseType::Int],
        &BaseType::Long,
    ),
];

impl Parser {
    pub fn verify_normal_simple_type(simple_type: &mut SimpleType) -> Result<(), String> {
        if simple_type.base_type.is_none() {
            return Err(format!("Type must have a base type"));
        }

        if simple_type.base_type.unwrap().is_floating() && simple_type.signedness.has_sign() {
            return Err(format!(
                "Floating point {} cannot have signedness",
                simple_type.base_type.unwrap()
            ));
        }

        if !simple_type.signedness.has_sign() && !simple_type.base_type.unwrap().is_floating() {
            simple_type.signedness = Signedness::Signed;
        }

        Ok(())
    }

    pub fn verify_tag_simple_type(simple_type: &SimpleType) -> Result<(), String> {
        if simple_type.base_type.is_some() {
            return Err(format!("Only qualifiers are allowed on tag type"));
        }

        if simple_type.is_inline_function {
            return Err(format!("Tag type can not be inline"));
        }

        if simple_type.signedness.has_sign() {
            return Err(format!("Tag type cannot have a signedness"));
        }

        Ok(())
    }

    pub fn parse_simple_type(&mut self) -> Result<SimpleType, String> {
        // let mut simple_type = SimpleType::default();
        let mut base_types = Vec::new();
        let mut qualifiers = Vec::new();
        let mut signedness = Vec::new();
        let mut storage_specifiers = Vec::new();
        let mut is_inline_function = false;
        let mut is_typedef = false;

        // let mut base_type
        while let Some(TokenTypes::DataType(data_type)) = self.lexer.peek() {
            match data_type {
                DataTypes::_Bool => base_types.push(BaseType::_Bool),
                DataTypes::Char => base_types.push(BaseType::Char),
                DataTypes::Short => base_types.push(BaseType::Short),
                DataTypes::Int => base_types.push(BaseType::Int),
                DataTypes::Long => base_types.push(BaseType::Long),
                DataTypes::Float => base_types.push(BaseType::Float),
                DataTypes::Double => base_types.push(BaseType::Double),
                DataTypes::Void => base_types.push(BaseType::Void),

                DataTypes::Signed => signedness.push(Signedness::Signed),
                DataTypes::Unsigned => signedness.push(Signedness::Unsigned),

                DataTypes::Register => storage_specifiers.push(Storage::Register),
                DataTypes::Auto => storage_specifiers.push(Storage::Auto),
                DataTypes::Static => storage_specifiers.push(Storage::Static),
                DataTypes::Extern => storage_specifiers.push(Storage::Extern),

                DataTypes::Const => qualifiers.push(Qualifiers::Const),
                DataTypes::Volatile => qualifiers.push(Qualifiers::Volatile),
                DataTypes::Restrict => qualifiers.push(Qualifiers::Restrict),

                DataTypes::Inline => is_inline_function = true,
                DataTypes::Typedef => is_typedef = true,

                _ => todo!(),
            }

            self.lexer.advance();
        }

        if storage_specifiers.len() > 1 {
            return Err(format!("Type cannot have multiple storage specifiers"));
        }

        base_types.sort_unstable();
        signedness.sort_unstable();
        qualifiers.sort_unstable();

        signedness.dedup();
        qualifiers.dedup();

        if signedness.len() > 1 {
            return Err(format!("Type cannot be signed and unsigned"));
        }

        let mut true_base_type: Option<BaseType> = None;
        for (test_type, base_type_result) in TYPE_LOOKUP {
            if base_types == test_type {
                true_base_type = Some(*base_type_result);
                break;
            }
        }

        if true_base_type.is_none() && signedness.len() == 1 {
            true_base_type = Some(BaseType::Int);
        } else if base_types.len() != 0 && true_base_type.is_none() {
            return Err(format!("Type given is not a valid combination"));
        }

        let signedness = match signedness.len() {
            1 => signedness[0],
            _ => Signedness::NoSign,
        };

        let storage = match storage_specifiers.len() {
            1 => Some(storage_specifiers[0]),
            _ => None,
        };

        Ok(SimpleType {
            base_type: true_base_type,
            storage,
            signedness,
            qualifiers,
            is_inline_function,
            is_typedef,
        })
    }
}
