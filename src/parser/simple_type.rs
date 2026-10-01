use std::fmt::Display;

use crate::{
    lexer::{language_features::DataTypes, lexer::TokenTypes},
    parser::parser::Parser,
};

#[derive(Clone, Debug, PartialEq, Default)]
pub struct SimpleType {
    pub base_type: DataTypes,
    pub is_inline: bool,
    pub storage: Option<DataTypes>,
    pub modifiers: Vec<DataTypes>,
    pub qualifiers: Vec<DataTypes>,
}

impl Display for SimpleType {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();

        let mut push_data_types = |data_type_list: &Vec<DataTypes>| {
            for data_type in data_type_list {
                output.push_str(&format!("{data_type} "));
            }
        };

        let mut data_type_list = Vec::new();
        if let Some(storage_specifier) = self.storage {
            data_type_list.push(storage_specifier);
        }

        data_type_list.extend(&self.qualifiers);
        data_type_list.extend(&self.modifiers);

        push_data_types(&data_type_list);

        output.push_str(&self.base_type.to_string());

        write!(display, "{output}")
    }
}

impl Parser {
    pub fn verify_simple_type(simple_type: &mut SimpleType) -> Result<(), String> {
        if simple_type.qualifiers.contains(&DataTypes::Restrict) {
            return Err(String::from("Restrict can only be used on pointers"));
        }

        let is_signed = simple_type.modifiers.contains(&DataTypes::Signed);
        let is_unsigned = simple_type.modifiers.contains(&DataTypes::Unsigned);

        if is_unsigned && is_signed {
            return Err(String::from("Type cannot contain both signed and unsigned"));
        }

        let is_short = simple_type.modifiers.contains(&DataTypes::Short);
        let is_long = simple_type.modifiers.contains(&DataTypes::Long);

        if is_short && is_long {
            return Err(String::from("Type cannot contain both short and long"));
        }

        if simple_type.base_type == DataTypes::NoType
            && (is_short || is_long || is_unsigned || is_signed)
        {
            simple_type.base_type = DataTypes::Int;
        }

        if (is_short || is_long) && simple_type.base_type == DataTypes::Float {
            return Err(String::from("Float cannot have modifier short or long"));
        }

        if is_short && simple_type.base_type == DataTypes::Double {
            return Err(String::from("Double cannot have modifier short"));
        }

        let remove_duplicates = |data_types: &mut Vec<DataTypes>| {
            data_types.sort_unstable();
            data_types.dedup();
        };

        remove_duplicates(&mut simple_type.qualifiers);

        Ok(())
    }

    pub fn parse_simple_type(&mut self) -> Result<SimpleType, String> {
        let mut simple_type = SimpleType::default();

        while let Some(TokenTypes::DataType(data_type)) = self.lexer.peek() {
            if data_type.is_qualifier() {
                simple_type.qualifiers.push(data_type);
            } else if data_type.is_modifier() {
                simple_type.modifiers.push(data_type);
            } else if data_type.is_storage_specifier() {
                if simple_type.storage.is_some() {
                    return Err(String::from(
                        "Type is not allowed to have more than one storage specifier",
                    ));
                }

                simple_type.storage = Some(data_type);
            } else if data_type.is_function_specifier() {
                simple_type.is_inline = true;
            } else {
                if !matches!(simple_type.base_type, DataTypes::NoType) {
                    return Err(String::from(
                        "Type is not allowed to have more than one base type",
                    ));
                }

                simple_type.base_type = data_type;
            }

            self.lexer.advance();
        }

        Ok(simple_type)
    }
}
