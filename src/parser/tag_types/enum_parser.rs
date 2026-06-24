use crate::lexer::language_features::AssignmentTypes;
use crate::lexer::language_features::KeywordTypes;
use crate::lexer::language_features::OperatorTypes;
use crate::lexer::lexer::{Lexer, TokenTypes};
use crate::parser::expression_parser::ExprNode;
use crate::parser::expression_parser::parse_expression;
use crate::parser::helper::pretty_clean_string;
use crate::parser::parser::GlobalNode;
use crate::parser::tag_types::helper::TagKeywordUsage;
use crate::parser::tag_types::helper::TagType;
use crate::parser::tag_types::helper::TagTypeMember;
use crate::parser::tag_types::helper::parse_tag_type_declaration;
use crate::parser::tag_types::helper::parse_tag_type_definition_and_vars;
use crate::parser::tag_types::helper::parse_tag_type_variable;
use crate::parser::tag_types::helper::tag_type_keyword_usage;

#[derive(Clone)]
pub struct EnumMember {
    pub name: String,
    pub value: Option<ExprNode>,
}

impl TagTypeMember for EnumMember {
    fn display_member(&self, indentation: usize) -> String {
        let indent_str = " ".repeat(indentation);
        let mut output = format!("{indent_str}(Member (Name {})", self.name);

        if let Some(enum_value) = &self.value {
            output.push_str(&format!(
                " (Value {})",
                pretty_clean_string(&enum_value.to_string())
            ));
        }

        output.push(')');
        output
    }
}

fn parse_enum_members(lexer: &mut Lexer) -> Result<Vec<EnumMember>, String> {
    let mut all_members = Vec::new();

    while let Some(token) = lexer.peek()
        && !matches!(token, TokenTypes::RCurlyBrace)
    {
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

        match next_token {
            TokenTypes::Operator(OperatorTypes::Comma) => {
                lexer.advance();
            }
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

    lexer.advance();

    Ok(all_members)
}

pub fn parse_enum_definition(lexer: &mut Lexer) -> Result<GlobalNode, String> {
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

    if members.len() == 0 {
        return Err(String::from(
            "Expected enum definition to have at least one variant",
        ));
    }

    Ok(GlobalNode::Enum(TagType {
        name,
        members,
        is_defined: true,
    }))
}

pub fn parse_enum_keyword(lexer: &mut Lexer) -> Result<Vec<GlobalNode>, String> {
    let usage = tag_type_keyword_usage(lexer)?;
    if matches!(usage, TagKeywordUsage::Variable) {
        return parse_tag_type_variable(lexer);
    }

    if matches!(usage, TagKeywordUsage::Definition) {
        return parse_tag_type_definition_and_vars(lexer, parse_enum_definition);
    }

    if matches!(usage, TagKeywordUsage::Declaration) {
        return Ok(vec![parse_tag_type_declaration(
            lexer,
            &KeywordTypes::Enum,
        )?]);
    }

    unreachable!()
}

// this doesn't need to be tested as heavily as most code is being reused from the struct parser
// and that code already has been verified
#[cfg(test)]
mod tests {
    use crate::parser::helper::run_tests;
    use crate::parser::parser::parse_program;

    #[test]
    fn enum_creation() {
        let test_cases = vec![
            (
                " enum Color {RED, GREEN, BLUE}; ",
                "
                (Enum Color (Members 
                    (Member (Name RED))
                    (Member (Name GREEN))
                    (Member (Name BLUE))
                ))
                ",
            ),
            ("enum Color c;", "(Variable (Name c (Enum Color)))"),
            (
                "enum Color { RED, GREEN, BLUE} c;",
                "
                (Enum Color (Members
                    (Member (Name RED))
                    (Member (Name GREEN))
                    (Member (Name BLUE))
                ))
                (Variable (Name c (Enum Color)))                    
                ",
            ),
            (
                "enum Numbers { ZERO = 0, ONE = 1, TWO = 2};",
                "
                (Enum Numbers (Members
                    (Member (Name ZERO) (Value (Num 0)))
                    (Member (Name ONE) (Value (Num 1)))
                    (Member (Name TWO) (Value (Num 2)))
                ))
                ",
            ),
            (
                "enum SignedValues {NEG = -1,ZERO = 0,POS = 1};",
                "
                (Enum SignedValues (Members
                    (Member (Name NEG) (Value (Unary (Op -) (Num 1))))
                    (Member (Name ZERO) (Value (Num 0)))
                    (Member (Name POS) (Value (Num 1)))
                ))  
                ",
            ),
            (
                "enum Expr {A = 1 + 2, B = A * 4, C = (B << 1)}; ",
                "
                (Enum Expr (Members
                    (Member (Name A) (Value
                        (Binary (Num 1) (Op +) (Num 2))))
                    (Member (Name B) (Value
                        (Binary (Var A) (Op *) (Num 4))))
                    (Member (Name C) (Value
                        (Binary (Var B) (Op <<) (Num 1))))
                ))                
                ",
            ),
            (
                "enum {A,B,C};",
                "
                (Enum (Members
                    (Member (Name A))
                    (Member (Name B))
                    (Member (Name C))
                ))                  
                ",
            ),
            ("enum Color;", "(Enum Color)"),
            (
                " enum Color {RED, GREEN, BLUE, }; ",
                "
                (Enum Color (Members
                    (Member (Name RED))
                    (Member (Name GREEN))
                    (Member (Name BLUE))
                ))
                ",
            ),
        ];

        run_tests(parse_program, test_cases);
    }
}
