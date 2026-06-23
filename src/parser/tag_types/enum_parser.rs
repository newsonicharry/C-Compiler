use crate::lexer::language_features::AssignmentTypes;
use crate::lexer::language_features::OperatorTypes;
use crate::lexer::language_features::{KeywordTypes, LiteralTypes};
use crate::lexer::lexer::{Lexer, TokenTypes};
use crate::parser::expression_parser::ExprNode;
use crate::parser::expression_parser::parse_expression;
use crate::parser::parser::parse_statement;
use crate::parser::parser::{GlobalNode, StatementNode};
use crate::parser::tag_types::helper::TagKeywordUsage;
use crate::parser::tag_types::helper::is_tag_type_keyword;
use crate::parser::tag_types::helper::parse_tag_type_qualifiers;
use crate::parser::tag_types::helper::tag_type_keyword_usage;
use crate::parser::tag_types::struct_parser::parse_vars_after_type;
use crate::parser::type_parser::{TypeNode, parse_type};
use std::fmt::Display;

pub struct Enum {
    pub name: Option<String>,
    pub members: Vec<EnumMember>,
}

impl Enum {
    pub fn display(indendation: usize) -> String {
        todo!()
    }
}

pub struct EnumMember {
    pub name: String,
    pub value: Option<ExprNode>,
}

impl Display for EnumMember {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = format!("(EnumMember (Name {})", self.name);

        if let Some(enum_value) = &self.value {
            output.push_str(&format!(" (Value {enum_value})"));
        }

        output.push(')');

        write!(display, "{output}")
    }
}

fn parse_enum_members(lexer: &mut Lexer) -> Result<Vec<EnumMember>, String> {
    let mut all_members = Vec::new();

    while let Some(token) = lexer.peek() {
        if matches!(token, TokenTypes::Operator(OperatorTypes::Comma)) {
            return Err(String::from("Unexpected comma in enum"));
        }

        let enum_name = lexer.expect_extract(|x| match x {
            TokenTypes::Identifier(name) => Some(name),
            _ => None,
        })?;

        let next_token = lexer.force_peek("Unexpected end to enum")?;

        let mut enum_value = None;

        if matches!(
            next_token,
            TokenTypes::Assignment(AssignmentTypes::SimpleAssignment)
        ) {
            lexer.advance();
            enum_value = Some(parse_expression(lexer, 3)?);
        }

        all_members.push(EnumMember {
            name: enum_name,
            value: enum_value,
        });

        let next_token = lexer.force_peek("Unexpected end to enum")?;
        lexer.advance();

        match next_token {
            TokenTypes::Operator(OperatorTypes::Comma) => {}
            TokenTypes::RCurlyBrace => {
                break;
            }
            unexpected_token => {
                return Err(format!(
                    "Unexpected token of type {unexpected_token}, expected comma or semicolon"
                ));
            }
        }
    }

    Ok(all_members)
}

pub fn parse_enum_definition(lexer: &mut Lexer) -> Result<Enum, String> {
    lexer.advance();

    let name = match lexer.peek() {
        Some(TokenTypes::Identifier(name)) => {
            lexer.advance();
            Some(name)
        }
        _ => None,
    };

    lexer.advance();

    let members = parse_enum_members(lexer)?;

    lexer.advance();

    Ok(Enum { name, members })
}

pub fn parse_enum_keyword(lexer: &mut Lexer) -> Result<Vec<GlobalNode>, String> {
    let usage = tag_type_keyword_usage(lexer)?;

    if matches!(usage, TagKeywordUsage::Definition) {
        let mut enum_and_vars = Vec::new();

        let mut var_qualifiers = parse_tag_type_qualifiers(lexer)?;

        let defined_enum = parse_enum_definition(lexer)?;
        let enum_name = defined_enum.name.clone();

        enum_and_vars.push(GlobalNode::Enum(defined_enum));

        var_qualifiers.extend(parse_tag_type_qualifiers(lexer)?);

        let struct_type = TypeNode::Enum {
            name: enum_name,
            qualifiers: var_qualifiers,
        };

        let defined_vars: Vec<GlobalNode> = parse_vars_after_type::<true>(lexer, &struct_type)?
            .iter()
            .map(|x| GlobalNode::Variable {
                expr_statement: x.clone(),
            })
            .collect();

        enum_and_vars.extend(defined_vars);

        return Ok(enum_and_vars);
    }

    todo!()
}
