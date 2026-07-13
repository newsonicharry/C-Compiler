use crate::lexer::language_features::KeywordTypes;
use crate::lexer::lexer::TokenTypes;
use crate::parser::nodes::GlobalNode;
use crate::parser::parser::Parser;
use crate::parser::tag_types::helper::TagKeywordUsage;
use crate::parser::tag_types::helper::TagTypeKind;
use crate::parser::tag_types::helper::TagTypeMember;
use crate::semantics::semantics::SemanticInfo;

impl Parser {
    pub fn parse_union_keyword(&mut self) -> Result<Vec<GlobalNode>, String> {
        let usage = self.tag_type_keyword_usage()?;

        if matches!(usage, TagKeywordUsage::Variable) {
            return self.parse_variable_statement();
        }
        if matches!(usage, TagKeywordUsage::Definition) {
            let parse_definition = |parser: &mut Parser| {
                return parser.parse_struct_or_union_definition(
                    TagTypeKind::Union,
                    Self::parse_union_member,
                );
            };
            return self.parse_tag_type_definition_and_vars(parse_definition);
        }

        // if its a declaration
        if matches!(usage, TagKeywordUsage::Declaration) {
            return Ok(vec![self.parse_tag_type_declaration(&KeywordTypes::Union)?]);
        }

        unreachable!()
    }

    fn parse_nested_tag_type(&mut self) -> Result<Option<Vec<TagTypeMember>>, String> {
        let mut all_members = Vec::new();

        let Some(items) = self.get_nested_member_if_some()? else {
            return Ok(None);
        };

        for item in items {
            let member = match item {
                GlobalNode::TagType(data) => TagTypeMember::TagType(data),

                GlobalNode::Initalizer { var_type, .. } => TagTypeMember::UnionMember {
                    item_type: var_type,
                    semantic_info: SemanticInfo::default(),
                },

                _ => unreachable!(),
            };

            all_members.push(member);
        }

        Ok(Some(all_members))
    }

    fn parse_union_member(&mut self) -> Result<Vec<TagTypeMember>, String> {
        if let Some(nested_members) = self.parse_nested_tag_type()? {
            return Ok(nested_members);
        }

        let member = self.parse_type()?;

        if member.has_invalid_tag_type_specifier() {
            return Err(String::from(
                "Unexpected tag type specifier found for union member",
            ));
        }

        self.lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

        let final_member = TagTypeMember::UnionMember {
            item_type: member,
            semantic_info: SemanticInfo::default(),
        };

        Ok(vec![final_member])
    }
}
#[cfg(test)]
mod tests {
    use crate::parser::{helper::run_tests, parser::Parser};

    #[test]
    fn union_creation() {
        let test_cases = vec![
            (
                "union U {int i;float f;};",
                "
                (Union U (Members
                    (Member (Name i (Type int)))
                    (Member (Name f (Type float)))
                ))
                ",
            ),
            (
                "union U {int i;float f;} u;",
                "
                (Union U (Members
                    (Member (Name i (Type int)))
                    (Member (Name f (Type float)))
                ))
                (Variable (Name u (Union U)))
                ",
            ),
            ("union U u;", "(Variable (Name u (Union U))) "),
            ("union U *p;", "(Variable (Name p (Ptr (Union U))))"),
            (
                "union {int i; float f;} u;",
                "
                (Union Anon-TagType-0 (Members
                    (Member (Name i (Type int)))
                    (Member (Name f (Type float)))
                ))
                (Variable (Name u (Union Anon-TagType-0)))
                ",
            ),
            (
                "const union U u;",
                "(Variable (Name u (Union U (Qualifiers const))))",
            ),
            (
                "
                union Outer {
                    int tag;
                    union {int i; float f;} data;
                };
                ",
                "
                (Union Outer (Members
                    (Member (Name tag (Type int)))
                    (Union Anon-TagType-0 (Members
                      (Member (Name i (Type int)))
                      (Member (Name f (Type float)))
                    ))
                    (Member (Name data (Union Anon-TagType-0)))
                ))
                ",
            ),
            (
                "
                union U {
                    struct {int x;int y;} point;
                    int raw[2];
                };
                ",
                "
                (Union U (Members
                    (Struct Anon-TagType-0 (Members
                        (Member (Name x (Type int)))
                        (Member (Name y (Type int)))
                    ))
                    (Member (Name point (Struct Anon-TagType-0)))
                    (Member (Name raw (Arr (Num 2) (Type int))))
                ))
                ",
            ),
            (
                "
                struct S {
                    int tag;
                    union {int i;float f;} value;
                };
                ",
                "
                (Struct S (Members
                    (Member (Name tag (Type int)))
                    (Union Anon-TagType-0 (Members
                        (Member (Name i (Type int)))
                        (Member (Name f (Type float)))
                    ))
                    (Member (Name value (Union Anon-TagType-0)))
                ))
                ",
            ),
            (
                " union U u = { 42 }; ",
                "
                (Variable (Name u (Union U))
                    (AggInit
                        (InitElement (Expr (Num 42)))))
            ",
            ),
        ];

        run_tests(Parser::parse_program, test_cases);
    }
}
