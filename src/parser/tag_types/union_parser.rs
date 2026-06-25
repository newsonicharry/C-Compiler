use crate::lexer::language_features::KeywordTypes;
use crate::lexer::lexer::{Lexer, TokenTypes};
use crate::parser::parser::{GlobalNode, StatementNode};
use crate::parser::tag_types::enum_parser::EnumMember;
use crate::parser::tag_types::helper::TagTypeMember;
use crate::parser::tag_types::helper::parse_tag_type_variable;
use crate::parser::tag_types::helper::{
    TagKeywordUsage, parse_tag_type_declaration, tag_type_keyword_usage,
};
use crate::parser::tag_types::helper::{TagType, parse_struct_or_union_definition};
use crate::parser::tag_types::helper::{
    get_nested_member_if_some, parse_tag_type_definition_and_vars,
};
use crate::parser::tag_types::struct_parser::StructMember;
use crate::parser::type_parser::{TypeNode, parse_type};

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

pub fn parse_union_keyword(lexer: &mut Lexer) -> Result<Vec<GlobalNode>, String> {
    let usage = tag_type_keyword_usage(lexer)?;

    if matches!(usage, TagKeywordUsage::Variable) {
        return parse_tag_type_variable(lexer);
    }

    if matches!(usage, TagKeywordUsage::Definition) {
        let parse_definition = |lexer: &mut Lexer| {
            return parse_struct_or_union_definition(lexer, parse_union_member);
        };
        return parse_tag_type_definition_and_vars(lexer, parse_definition);
    }

    // if its a declaration
    if matches!(usage, TagKeywordUsage::Declaration) {
        return Ok(vec![parse_tag_type_declaration(
            lexer,
            &KeywordTypes::Struct,
        )?]);
    }

    unreachable!()
}

fn parse_nested_tag_type(lexer: &mut Lexer) -> Result<Option<Vec<UnionMember>>, String> {
    let mut all_members = Vec::new();

    let Some(items) = get_nested_member_if_some(lexer)? else {
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

fn parse_union_member(lexer: &mut Lexer) -> Result<Vec<UnionMember>, String> {
    if let Some(nested_members) = parse_nested_tag_type(lexer)? {
        return Ok(nested_members);
    }

    let member = parse_type(lexer)?;

    lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

    let final_member = UnionMember::NormalType { item_type: member };

    Ok(vec![final_member])
}

#[cfg(test)]
mod tests {
    use crate::parser::helper::run_tests;
    use crate::parser::parser::parse_program;

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

        run_tests(parse_program, test_cases);
    }
}
