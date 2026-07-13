use crate::lexer::language_features::AssignmentTypes;
use crate::lexer::language_features::KeywordTypes;
use crate::lexer::language_features::OperatorTypes;
use crate::lexer::lexer::TokenTypes;
use crate::parser::nodes::GlobalNode;
use crate::parser::parser::Parser;
use crate::parser::tag_types::helper::TagKeywordUsage;
use crate::parser::tag_types::helper::TagTypeData;
use crate::parser::tag_types::helper::TagTypeKind;
use crate::parser::tag_types::helper::TagTypeMember;
use crate::semantics::semantics::SemanticInfo;

impl Parser {
    fn parse_enum_members(&mut self) -> Result<Vec<TagTypeMember>, String> {
        let mut all_members = Vec::new();
        while let Some(token) = self.lexer.peek()
            && !matches!(token, TokenTypes::RCurlyBrace)
        {
            if matches!(token, TokenTypes::Operator(OperatorTypes::Comma)) {
                return Err(String::from("Unexpected comma in enum"));
            }

            let enum_name = self.lexer.expect_extract(|x| match x {
                TokenTypes::Identifier(name) => Some(name),
                _ => None,
            })?;

            let next_token = self.lexer.force_peek("Unexpected end to enum")?;

            let mut enum_value = None;

            if matches!(
                next_token,
                TokenTypes::Assignment(AssignmentTypes::SimpleAssignment)
            ) {
                self.lexer.advance();
                enum_value = Some(self.parse_expression(3)?);
            }
            all_members.push(TagTypeMember::EnumMember {
                name: enum_name,
                value: enum_value,
                semantic_info: SemanticInfo::default(),
            });

            let next_token = self.lexer.force_peek("Unexpected end to enum")?;

            match next_token {
                TokenTypes::Operator(OperatorTypes::Comma) => {
                    self.lexer.advance();
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

        self.lexer.advance();

        Ok(all_members)
    }

    pub fn parse_enum_definition(&mut self) -> Result<TagTypeData, String> {
        self.lexer.advance();

        let name = match self.lexer.peek() {
            Some(TokenTypes::Identifier(name)) => {
                self.lexer.advance();
                Some(name)
            }
            _ => None,
        };

        self.lexer.advance();
        let members = self.parse_enum_members()?;

        if members.len() == 0 {
            return Err(String::from(
                "Expected enum definition to have at least one variant",
            ));
        }

        Ok(TagTypeData {
            kind: TagTypeKind::Enum,
            name,
            members,
            is_defined: true,
            semantic_info: SemanticInfo::default(),
        })
    }

    pub fn parse_enum_keyword(&mut self) -> Result<Vec<GlobalNode>, String> {
        let usage = self.tag_type_keyword_usage()?;
        if matches!(usage, TagKeywordUsage::Variable) {
            return self.parse_variable_statement();
        }

        if matches!(usage, TagKeywordUsage::Definition) {
            return self.parse_tag_type_definition_and_vars(Self::parse_enum_definition);
        }

        if matches!(usage, TagKeywordUsage::Declaration) {
            return Ok(vec![self.parse_tag_type_declaration(&KeywordTypes::Enum)?]);
        }

        unreachable!()
    }
}
// this doesn't need to be tested as heavily as most code is being reused from the struct parser
// and that code already has been verified
#[cfg(test)]
mod tests {
    use crate::parser::{helper::run_tests, parser::Parser};

    #[test]
    fn enum_creation() {
        let test_cases = vec![
            (
                " enum Color {RED, GREEN, BLUE}; ",
                "
                (Enum Color (Members
                    (Member RED)
                    (Member GREEN)
                    (Member BLUE)
                ))
                ",
            ),
            ("enum Color c;", "(Variable (Name c (Enum Color)))"),
            (
                "enum Color { RED, GREEN, BLUE} c;",
                "
                (Enum Color (Members
                    (Member RED)
                    (Member GREEN)
                    (Member BLUE)
                ))
                (Variable (Name c (Enum Color)))
                ",
            ),
            (
                "enum Numbers { ZERO = 0, ONE = 1, TWO = 2};",
                "
                (Enum Numbers (Members
                    (Member ZERO (Value (Num 0)))
                    (Member ONE (Value (Num 1)))
                    (Member TWO (Value (Num 2)))
                ))
                ",
            ),
            (
                "enum SignedValues {NEG = -1,ZERO = 0,POS = 1};",
                "
                (Enum SignedValues (Members
                    (Member NEG (Value (Unary (Op -) (Num 1))))
                    (Member ZERO (Value (Num 0)))
                    (Member POS (Value (Num 1)))
                ))
                ",
            ),
            (
                "enum Expr {A = 1 + 2, B = A * 4, C = (B << 1)}; ",
                "
                (Enum Expr (Members
                    (Member A (Value (Binary (Num 1) (Op +) (Num 2))))
                    (Member B (Value (Binary (Var A) (Op *) (Num 4))))
                    (Member C (Value (Binary (Var B) (Op <<) (Num 1))))
                ))
                ",
            ),
            ("enum {A,B,C};", ""), // should be empty as it doesnt define anything
            ("enum Color;", "(Enum Color)"),
            (
                " enum Color {RED, GREEN, BLUE, }; ",
                "
                (Enum Color (Members
                    (Member RED)
                    (Member GREEN)
                    (Member BLUE)
                ))
                ",
            ),
        ];
        run_tests(Parser::parse_program, test_cases);
    }
}
