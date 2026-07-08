use crate::lexer::language_features::KeywordTypes;
use crate::lexer::lexer::TokenTypes;
use crate::parser::nodes::GlobalNode;
use crate::parser::parser::Parser;
use crate::parser::tag_types::enum_parser::EnumMember;
use crate::parser::tag_types::helper::TagKeywordUsage;
use crate::parser::tag_types::helper::TagType;
use crate::parser::tag_types::helper::TagTypeMember;
use crate::parser::tag_types::struct_parser::StructMember;
use crate::parser::type_parser::TypeNode;

#[derive(Clone)]
pub enum UnionMember {
    NormalType { item_type: TypeNode },
    DefinedStruct(TagType<StructMember>),
    DefinedEnum(TagType<EnumMember>),
    DefinedUnion(TagType<UnionMember>),
}

impl TagTypeMember for UnionMember {
    fn display_member(&self, indentation: usize) -> String {
        let mut output = String::new();
        let indent_str = " ".repeat(indentation);

        match self {
            UnionMember::NormalType { item_type } => {
                output += &format!("{indent_str}(Member {}", item_type);
            }

            UnionMember::DefinedStruct(data) => {
                output += &data.display(indentation + 2);
            }

            UnionMember::DefinedEnum(data) => {
                output += &data.display(indentation + 2);
            }

            UnionMember::DefinedUnion(data) => {
                output += &data.display(indentation + 2);
            }
        }

        output.push_str(")");

        output
    }
}

impl Parser {
    pub fn parse_union_keyword(&mut self) -> Result<Vec<GlobalNode>, String> {
        let usage = self.tag_type_keyword_usage()?;

        if matches!(usage, TagKeywordUsage::Variable) {
            return self.parse_variable_statement();
        }

        if matches!(usage, TagKeywordUsage::Definition) {
            let parse_definition = |parser: &mut Parser| {
                return parser.parse_struct_or_union_definition(Self::parse_union_member);
            };
            return self.parse_tag_type_definition_and_vars(parse_definition);
        }

        // if its a declaration
        if matches!(usage, TagKeywordUsage::Declaration) {
            return Ok(vec![self.parse_tag_type_declaration(&KeywordTypes::Union)?]);
        }

        unreachable!()
    }

    fn parse_nested_tag_type(&mut self) -> Result<Option<Vec<UnionMember>>, String> {
        let mut all_members = Vec::new();

        let Some(items) = self.get_nested_member_if_some()? else {
            return Ok(None);
        };

        for item in items {
            let member = match item {
                GlobalNode::Struct(data) => UnionMember::DefinedStruct(data),
                GlobalNode::Enum(data) => UnionMember::DefinedEnum(data),
                GlobalNode::Union(data) => UnionMember::DefinedUnion(data),

                GlobalNode::Initalizer { var_type, .. } => UnionMember::NormalType {
                    item_type: var_type,
                },

                _ => unreachable!(),
            };

            all_members.push(member);
        }

        Ok(Some(all_members))
    }

    fn parse_union_member(&mut self) -> Result<Vec<UnionMember>, String> {
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

        let final_member = UnionMember::NormalType { item_type: member };

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
                (Union (Members
                    (Member (Name i (Type int)))
                    (Member (Name f (Type float)))
                ))
                (Variable (Name u (Union)))   
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
                    (Union (Members
                      (Member (Name i (Type int)))
                      (Member (Name f (Type float)))
                    )))
                    (Member (Name data (Union)))
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
                    (Struct (Members
                        (Member (Name x (Type int)))
                        (Member (Name y (Type int)))
                    )))
                    (Member (Name point (Struct)))
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
                    (Union (Members
                        (Member (Name i (Type int)))
                        (Member (Name f (Type float)))
                    )))
                    (Member (Name value (Union)))
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
