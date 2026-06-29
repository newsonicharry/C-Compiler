use crate::lexer::language_features::AssignmentTypes;
use crate::lexer::language_features::DataTypes;
use crate::lexer::language_features::KeywordTypes;
use crate::lexer::language_features::OperatorTypes;
use crate::lexer::lexer::{Lexer, TokenTypes};
use crate::parser::expression_parser::parse_expression;
use crate::parser::parser::GlobalNode;
use crate::parser::parser::parse_variable_statement;
use crate::parser::tag_types::enum_parser::EnumMember;
use crate::parser::tag_types::enum_parser::parse_enum_keyword;
use crate::parser::tag_types::struct_parser::StructMember;
use crate::parser::tag_types::struct_parser::parse_struct_keyword;
use crate::parser::tag_types::union_parser::UnionMember;
use crate::parser::tag_types::union_parser::parse_union_keyword;
use crate::parser::type_parser::TypeNode;
use crate::parser::type_parser::parse_type;

pub trait TagTypeMember {
    fn display_member(&self, indentation: usize) -> String;
}

pub trait ToGlobal {
    fn to_global(&self) -> GlobalNode;
}

#[derive(Clone)]
pub struct TagType<T> {
    pub is_defined: bool,
    pub name: Option<String>,
    pub members: Vec<T>,
}

impl ToGlobal for TagType<StructMember> {
    fn to_global(&self) -> GlobalNode {
        GlobalNode::Struct(self.clone())
    }
}

impl ToGlobal for TagType<UnionMember> {
    fn to_global(&self) -> GlobalNode {
        GlobalNode::Union(self.clone())
    }
}

impl<T: TagTypeMember> TagType<T> {
    fn display_generic(&self, tag_type_name: &str, indentation: usize) -> String {
        let mut output = String::new();
        let indent_str = " ".repeat(indentation);
        output.push_str(&format!("{indent_str}({tag_type_name}"));
        if let Some(name) = self.name.clone() {
            output.push_str(&format!(" {name}"));
        }

        if self.is_defined {
            output.push_str(&format!(" (Members"));

            for member in &self.members {
                output.push_str(&format!("\n{}", member.display_member(indentation + 2)));
            }

            if !self.members.is_empty() {
                output.push_str(&format!("\n{indent_str}"));
            }
            output.push_str(")");
        }

        output.push_str(")");

        output
    }
}

impl TagType<EnumMember> {
    pub fn display(&self, indentation: usize) -> String {
        self.display_generic("Enum", indentation)
    }
}

impl TagType<StructMember> {
    pub fn display(&self, indentation: usize) -> String {
        self.display_generic("Struct", indentation)
    }
}

impl TagType<UnionMember> {
    pub fn display(&self, indentation: usize) -> String {
        self.display_generic("Union", indentation)
    }
}

#[derive(Debug)]
pub enum TagKeywordUsage {
    Definition,
    Declaration,
    Variable,
}

/// Parses the qualifiers of a tag type after its been defined
/// (e.g struct Point{int x;} const volatile p, q;) where the const volatile portion is parsed
pub fn parse_tag_type_qualifiers(lexer: &mut Lexer) -> Result<Vec<DataTypes>, String> {
    let mut qualifiers = Vec::new();
    while let Some(TokenTypes::DataType(data_type)) = lexer.peek() {
        if data_type.is_qualifier() || data_type.is_storage_specifier() {
            qualifiers.push(data_type);
            lexer.advance();
            continue;
        }

        return Err(format!(
            "Expected data type after tag type to be a qualifier, got {data_type}"
        ));
    }

    Ok(qualifiers)
}

/// Determines how the tag type keyword is being used
/// This could be eihter as a defintion, declaration or variable
pub fn tag_type_keyword_usage(lexer: &mut Lexer) -> Result<TagKeywordUsage, String> {
    lexer.set_flag();

    // the qualifiers don't matter we just want to skip them here
    parse_tag_type_qualifiers(lexer)?;

    lexer.advance(); // move past the tag type keyword

    // Move past the tag type name if it exists
    if let Some(TokenTypes::Identifier(_)) = lexer.peek() {
        lexer.next();
    }

    parse_tag_type_qualifiers(lexer)?;

    let next_token = lexer.force_peek("Expected next token in tag type definition, got nothing")?;

    // make sure we don't mess up the parsing for our parsing functions
    lexer.recede_to_flag();

    // if its a variable
    // includes left parenthesis and start because it could be a function pointer or pointer
    if matches!(next_token, TokenTypes::Identifier(_))
        || matches!(next_token, TokenTypes::Operator(OperatorTypes::LParen))
        || matches!(next_token, TokenTypes::Operator(OperatorTypes::Star))
    {
        return Ok(TagKeywordUsage::Variable);
    }

    if matches!(next_token, TokenTypes::LCurlyBrace) {
        return Ok(TagKeywordUsage::Definition);
    }

    if matches!(next_token, TokenTypes::Semicolon) {
        return Ok(TagKeywordUsage::Declaration);
    }

    Err(String::from(&format!(
        "Unexpected next token {next_token}, expected tag type variable, definition or declaration",
    )))
}

/// Determines if a sequence of tokens uses a certain tag type
/// Used within the main parser to determine if it should go to the struct parser
pub fn is_tag_type_keyword(lexer: &mut Lexer, keyword: &KeywordTypes) -> Result<bool, String> {
    lexer.set_flag();
    let _ = parse_tag_type_qualifiers(lexer);

    let curr_token = lexer.force_peek("Expected next token, got nothing")?;
    if curr_token == TokenTypes::Keyword(*keyword) {
        lexer.recede_to_flag();
        return Ok(true);
    }

    if matches!(curr_token, TokenTypes::Keyword(_)) {
        lexer.recede_to_flag();
        return Ok(false);
    }

    let parsed_type = parse_type(lexer)?;

    lexer.recede_to_flag();

    if parsed_type.contains_tag_type(keyword) {
        return Ok(true);
    }

    Ok(false)
}

/// Parses tag types declarations
/// (e.g. Struct Point; )
pub fn parse_tag_type_declaration(
    lexer: &mut Lexer,
    keyword: &KeywordTypes,
) -> Result<GlobalNode, String> {
    // qualifiers in a declaration are not used
    parse_tag_type_qualifiers(lexer)?;

    // move past the struct keyword
    lexer.advance();

    let Some(TokenTypes::Identifier(tag_type_name)) = lexer.peek() else {
        unreachable!()
    };

    lexer.advance();

    parse_tag_type_qualifiers(lexer)?;

    let declared_tag_type = match keyword {
        KeywordTypes::Struct => GlobalNode::Struct(TagType {
            is_defined: false,
            name: Some(tag_type_name),
            members: vec![],
        }),

        KeywordTypes::Enum => GlobalNode::Enum(TagType {
            is_defined: false,
            name: Some(tag_type_name),
            members: vec![],
        }),

        _ => unreachable!(),
    };

    lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

    Ok(declared_tag_type)
}

pub fn parse_tag_type_variable(lexer: &mut Lexer) -> Result<Vec<GlobalNode>, String> {
    parse_variable_statement(lexer)
}

pub fn get_nested_member_if_some(lexer: &mut Lexer) -> Result<Option<Vec<GlobalNode>>, String> {
    let mut items = Vec::new();
    // disgusting but there really is not a better option
    if is_tag_type_keyword(lexer, &KeywordTypes::Struct)? {
        items.extend(parse_struct_keyword(lexer)?);
    } else if is_tag_type_keyword(lexer, &KeywordTypes::Union)? {
        items.extend(parse_union_keyword(lexer)?);
    } else if is_tag_type_keyword(lexer, &KeywordTypes::Enum)? {
        items.extend(parse_enum_keyword(lexer)?);
    } else {
        return Ok(None);
    }

    Ok(Some(items))
}

// helpers
fn token_is_variable_type(token: &TokenTypes) -> bool {
    match token {
        TokenTypes::Identifier(_) => true,
        TokenTypes::Operator(op_type) => match op_type {
            OperatorTypes::LParen | OperatorTypes::Star => true,
            _ => false,
        },

        _ => false,
    }
}

fn update_var(struct_type: &TypeNode, lexer: &mut Lexer) -> Result<TypeNode, String> {
    let mut var_type = parse_type(lexer)?;
    var_type.set_most_nested_held_value(struct_type);
    Ok(var_type)
}

/// Parses the indentifiers after a variable
/// This exists because a variable can define multiple different vars using the comma operator
/// (e.g. int x = 10, y = 20, z;) where the funtion runs after the int x = 10 is parsed
pub fn parse_vars_after_type<const IS_STRUCT: bool>(
    lexer: &mut Lexer,
    struct_type: &TypeNode,
) -> Result<Vec<GlobalNode>, String> {
    let mut var_type;
    let mut all_vars = Vec::new();

    for i in 0.. {
        let next_token = lexer.force_peek("Expected end of variable definition, got nothing")?;

        // a struct extra vars don't have to start after a comma, it just starts after the right curly brace
        // or after the qulifiers after the left curly brace
        if IS_STRUCT && i == 0 && token_is_variable_type(&next_token) {
            var_type = update_var(&struct_type, lexer)?;
        }
        // Could be another variable assigned after the original one
        else if matches!(next_token, TokenTypes::Operator(OperatorTypes::Comma)) {
            lexer.advance();
            var_type = update_var(&struct_type, lexer)?;
        } else if matches!(next_token, TokenTypes::Semicolon) {
            break;
        } else {
            // Variable assignment can only end wiht a comma or semi colon
            return Err(format!(
                "Expected comma or semicolon after variable definition, got token of type {next_token}"
            ));
        }

        let next_token = lexer.force_peek("Unexpected end of variable definition, got nothing")?;
        let final_var;

        // its a definition
        if matches!(
            next_token,
            TokenTypes::Assignment(AssignmentTypes::SimpleAssignment)
        ) {
            lexer.advance();

            final_var = GlobalNode::Initalizer {
                var_type: var_type.clone(),
                r_value: Some(parse_expression(lexer, 3)?),
            };
        }
        // its a declaration
        else {
            final_var = GlobalNode::Initalizer {
                var_type: var_type.clone(),
                r_value: None,
            };
        }

        all_vars.push(final_var);
    }

    lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

    Ok(all_vars)
}

/// Parses struct or union definitions
/// These include stuct with a left and right curly brace that may or may not include members
/// This does not account for variables defined subsequently with the struct
pub fn parse_struct_or_union_definition<F, T>(
    lexer: &mut Lexer,
    parse_member: F,
) -> Result<GlobalNode, String>
where
    F: Fn(&mut Lexer) -> Result<Vec<T>, String>,
    TagType<T>: ToGlobal,
{
    lexer.advance(); // move past the struct

    let name = match lexer.peek() {
        Some(TokenTypes::Identifier(name)) => {
            lexer.advance();
            Some(name)
        }
        _ => None,
    };

    lexer.advance();

    let mut members = Vec::new(); // literally everything else including regular struct variables

    while !matches!(lexer.peek(), Some(TokenTypes::RCurlyBrace)) {
        let final_member = parse_member(lexer)?;

        // becuase multiple semicolons are allowed for some reason
        while let Some(TokenTypes::Semicolon) = lexer.peek() {
            lexer.advance();
        }

        members.extend(final_member);
    }

    lexer.advance();

    let final_struct_or_union = TagType {
        is_defined: true,
        name: name.clone(),
        members,
    };

    Ok(final_struct_or_union.to_global())
}

pub fn parse_tag_type_definition_and_vars<F>(
    lexer: &mut Lexer,
    parse_definition: F,
) -> Result<Vec<GlobalNode>, String>
where
    F: Fn(&mut Lexer) -> Result<GlobalNode, String>,
{
    let mut enum_and_vars = Vec::new();

    let mut var_qualifiers = parse_tag_type_qualifiers(lexer)?;

    let defined_enum = parse_definition(lexer)?;

    enum_and_vars.push(defined_enum.clone());

    var_qualifiers.extend(parse_tag_type_qualifiers(lexer)?);

    let tag_type_type = match defined_enum {
        GlobalNode::Enum(data) => TypeNode::Enum {
            name: data.name.clone(),
            qualifiers: var_qualifiers,
        },

        GlobalNode::Struct(data) => TypeNode::Struct {
            name: data.name,
            qualifiers: var_qualifiers,
        },

        GlobalNode::Union(data) => TypeNode::Union {
            name: data.name,
            qualifiers: var_qualifiers,
        },

        _ => unreachable!(),
    };

    let defined_vars: Vec<GlobalNode> = parse_vars_after_type::<true>(lexer, &tag_type_type)?;

    enum_and_vars.extend(defined_vars);

    return Ok(enum_and_vars);
}
