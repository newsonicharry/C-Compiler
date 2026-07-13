use crate::lexer::language_features::OperatorTypes;
use crate::lexer::language_features::{KeywordTypes, LiteralTypes};
use crate::lexer::lexer::TokenTypes;
use crate::parser::nodes::GlobalNode;
use crate::parser::parser::Parser;
use crate::parser::tag_types::helper::TagTypeKind;
use crate::parser::tag_types::helper::{TagKeywordUsage, TagTypeMember};
use crate::semantics::semantics::SemanticInfo;

impl Parser {
    /// Parses anything that uses a struct keyword
    /// This includes struct definitions, declarations and variables
    pub fn parse_struct_keyword(&mut self) -> Result<Vec<GlobalNode>, String> {
        let usage = self.tag_type_keyword_usage()?;
        if matches!(usage, TagKeywordUsage::Variable) {
            return self.parse_variable_statement();
        }

        if matches!(usage, TagKeywordUsage::Definition) {
            let parse_definition = |parser: &mut Parser| {
                return parser.parse_struct_or_union_definition(
                    TagTypeKind::Struct,
                    Self::parse_struct_member,
                );
            };
            return self.parse_tag_type_definition_and_vars(parse_definition);
        }

        // if its a declaration
        if matches!(usage, TagKeywordUsage::Declaration) {
            return Ok(vec![
                self.parse_tag_type_declaration(&KeywordTypes::Struct)?,
            ]);
        }

        unreachable!()
    }

    fn parse_nested_struct_tag_type(&mut self) -> Result<Option<Vec<TagTypeMember>>, String> {
        let mut all_members = Vec::new();

        let Some(items) = self.get_nested_member_if_some()? else {
            return Ok(None);
        };

        for item in items {
            let member = match item {
                GlobalNode::TagType(data) => TagTypeMember::TagType(data),
                GlobalNode::Initalizer { var_type, .. } => TagTypeMember::StructMember {
                    item_type: var_type,
                    bit_field: None,
                    semantic_info: SemanticInfo::default(),
                },

                _ => unreachable!(),
            };

            all_members.push(member);
        }

        Ok(Some(all_members))
    }

    fn parse_struct_member(&mut self) -> Result<Vec<TagTypeMember>, String> {
        if let Some(nested_members) = self.parse_nested_struct_tag_type()? {
            return Ok(nested_members);
        }

        let member = self.parse_type()?;

        if member.has_invalid_tag_type_specifier() {
            return Err(String::from(
                "Unexpected tag type specifier found for struct member",
            ));
        }

        let Some(next_token) = self.lexer.peek() else {
            return Err(String::from(
                "Expected either a semicolon or colon at the end of struct member, got nothing",
            ));
        };

        let mut bit_field = None;

        if matches!(next_token, TokenTypes::Operator(OperatorTypes::Colon)) {
            self.lexer.advance();

            bit_field = Some(self.lexer.expect_extract(|x| match x {
                TokenTypes::Literal(LiteralTypes::Integer(integer)) => Some(integer.value as u64),
                _ => None,
            })?);
        }

        self.lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

        let final_member = TagTypeMember::StructMember {
            item_type: member,
            bit_field,
            semantic_info: SemanticInfo::default(),
        };

        Ok(vec![final_member])
    }
}
#[cfg(test)]
mod tests {
    use crate::parser::{helper::run_tests, parser::Parser};

    #[test]
    fn struct_creation() {
        let test_cases = vec![
            ("struct Point;", "(Struct Point)"),
            ("const struct Point;", "(Struct Point)"),
            ("struct Point const;", "(Struct Point)"),
            ("struct {};", ""), // should be empty, ignored by the ast
            ("struct Point{};", "(Struct Point (Members))"),
            (
                "struct Point{int x; int y;};",
                "(Struct Point (Members
                  (Member (Name x (Type int)))
                  (Member (Name y (Type int)))
                ))",
            ),
            ("struct {int x; int y;};", ""),
            (
                "struct Point{int x : 3; int y : 2;};",
                "(Struct Point (Members
                  (Member (Name x (Type int)) (Bitfield 3))
                  (Member (Name y (Type int)) (Bitfield 2))
                ))",
            ),
        ];

        run_tests(Parser::parse_program, test_cases);
    }

    #[test]
    fn struct_var() {
        let test_cases = vec![
            (r#"struct Person p;"#, "(Variable (Name p (Struct Person)))"),
            (
                r#"struct Person p = {"Bob", 25};"#,
                "
                (Variable (Name p (Struct Person)) (AggInit
                        (AggInit
                            (InitElement (Expr (Char B)))
                            (InitElement (Expr (Char o)))
                            (InitElement (Expr (Char b)))
                            (InitElement (Expr (Char \\0)))
                        )
                        (InitElement (Expr (Num 25)))
                ))
                    
                ",
            ),
            (
                r#"struct Person p = {.name = "Bob", .age = 25};"#,
                "
                (Variable (Name p (Struct Person)) (AggInit
                        (Member (Var name (Expr (Str Bob))))
                        (Member (Var age (Expr (Num 25))))
                ))
                ",
            ),
            (
                r#"struct Person p = (struct Person){"Bob", 25};"#,
                "
                (Variable (Name p (Struct Person)) (Cast (Struct Person)
                    (AggInit
                        (AggInit
                            (InitElement (Expr (Char B)))
                            (InitElement (Expr (Char o)))
                            (InitElement (Expr (Char b)))
                            (InitElement (Expr (Char \\0)))
                        )
                        (InitElement (Expr (Num 25)))
                    )
                ))
                ",
            ),
            (
                r#"struct person *p;"#,
                "(Variable (Name p (Ptr (Struct person))))",
            ),
        ];

        run_tests(Parser::parse_program, test_cases);
    }

    #[test]
    fn struct_multi_var() {
        let test_cases = vec![
            (
                "struct Point p = {1,2}, q = {3,4};",
                "
                (Variable (Name p (Struct Point)) (AggInit
                    (InitElement (Expr (Num 1)))
                    (InitElement (Expr (Num 2)))
                ))                
                (Variable (Name q (Struct Point)) (AggInit
                    (InitElement (Expr (Num 3)))
                    (InitElement (Expr (Num 4)))
                ))               
                ",
            ),
            (
                "struct Point p = 1, q = 2;",
                "
                (Variable (Name p (Struct Point)) (Num 1))
                (Variable (Name q (Struct Point)) (Num 2))
                ",
            ),
            (
                "struct Point p = 1, q;",
                "
                (Variable (Name p (Struct Point)) (Num 1))
                (Variable (Name q (Struct Point)))
                ",
            ),
            (
                "struct Point p, q;",
                "
                (Variable (Name p (Struct Point)))
                (Variable (Name q (Struct Point)))
                ",
            ),
        ];

        run_tests(Parser::parse_program, test_cases);
    }

    #[test]
    fn struct_creation_multi_var() {
        let test_cases = vec![
            (
                "struct Point{int x;} p = {0}, q;",
                "
                (Struct Point (Members (Member (Name x (Type int)))))
                (Variable (Name p (Struct Point)) (AggInit
                    (InitElement (Expr (Num 0)))
                ))
                (Variable (Name q (Struct Point)))
                ",
            ),
            (
                "struct Point{int x; int y;} p = {1,2}, q = {3, 4};",
                "
                (Struct Point (Members (Member (Name x (Type int))) (Member (Name y (Type int)))))
                (Variable (Name p (Struct Point)) (AggInit
                    (InitElement (Expr (Num 1)))
                    (InitElement (Expr (Num 2)))
                ))
                (Variable (Name q (Struct Point)) (AggInit
                    (InitElement (Expr (Num 3)))
                    (InitElement (Expr (Num 4)))
                ))
                ",
            ),
            (
                "struct Point{int x; int y;} p = {5}; ",
                "
                (Struct Point (Members (Member (Name x (Type int))) (Member (Name y (Type int)))))
                (Variable (Name p (Struct Point)) (AggInit
                    (InitElement (Expr (Num 5)))
                ))
                ",
            ),
            (
                "struct Point {int x; int y;} p = {.y = 7};",
                "
                (Struct Point (Members (Member (Name x (Type int))) (Member (Name y (Type int)))))
                (Variable (Name p (Struct Point)) (AggInit
                    (Member (Var y (Expr (Num 7))))
                ))
                ",
            ),
            (
                "struct Point {int x;} p = {1}, *ptr = &p;",
                "
                (Struct Point (Members (Member (Name x (Type int)))))
                (Variable (Name p (Struct Point)) (AggInit
                    (InitElement (Expr (Num 1)))))
                (Variable (Name ptr (Ptr (Struct Point)))
                    (Unary
                        (Op &)
                        (Var p)))
                ",
            ),
            (
                "struct Point{int x;} points[2] = {{1}, {2}};",
                "
                (Struct Point (Members (Member (Name x (Type int)))))
                (Variable (Name points (Arr (Num 2) (Struct Point)))
                    (AggInit
                        (AggInit
                            (InitElement (Expr (Num 1))))
                        (AggInit
                            (InitElement (Expr (Num 2))))))
                ",
            ),
            (
                "struct Point{int x;} p, q;",
                "
                (Struct Point (Members (Member (Name x (Type int)))))
                (Variable (Name p (Struct Point)))
                (Variable (Name q (Struct Point)))
            ",
            ),
        ];

        run_tests(Parser::parse_program, test_cases);
    }

    #[test]
    fn struct_qualifiers() {
        let test_cases = vec![
            (
                "struct Point{ int x; } const p = {1};",
                "
                (Struct Point (Members (Member (Name x (Type int)))))
                (Variable (Name p (Struct Point (Qualifiers const)))
                    (AggInit
                        (InitElement (Expr (Num 1)))))                    
                ",
            ),
            (
                "const struct Point{ int x; } p = {1};",
                "
                (Struct Point (Members (Member (Name x (Type int)))))
                (Variable (Name p (Struct Point (Qualifiers const)))
                    (AggInit
                        (InitElement(Expr (Num 1)))))
                    
                ",
            ),
            (
                "struct Point{ int x; } volatile p;",
                "
                (Struct Point (Members (Member (Name x (Type int)))))
                (Variable (Name p (Struct Point (Qualifiers volatile))))   
                ",
            ),
            (
                "struct Point{ int x; } const volatile p = {0}, q = {1};",
                "
                (Struct Point (Members (Member (Name x (Type int)))))
                (Variable (Name p (Struct Point (Qualifiers const volatile)))
                    (AggInit
                        (InitElement (Expr (Num 0)))))
                (Variable (Name q (Struct Point (Qualifiers const volatile)))
                    (AggInit
                      (InitElement (Expr (Num 1)))))
                ",
            ),
            (
                "struct Point{ int x; } *restrict p;",
                "
                (Struct Point (Members (Member (Name x (Type int)))))
                (Variable (Name p (Ptr restrict (Struct Point))))    
                ",
            ),
            (
                "struct Point{ int x; } const points[4] = {{0}};",
                "
                (Struct Point (Members (Member (Name x (Type int)))))
                (Variable (Name points (Arr (Num 4) (Struct Point (Qualifiers const))))
                    (AggInit
                        (AggInit
                            (InitElement (Expr (Num 0))))))
                ",
            ),
            (
                "extern struct Point{ int x; } p;",
                "
                (Struct Point (Members (Member (Name x (Type int)))))
                (Variable (Name p (Struct Point (Qualifiers extern))))
                ",
            ),
            (
                "struct point { int x; } (*fp)(void);",
                "
                (Struct point (Members (Member (Name x (Type int)))))
                (Variable (Name fp (FuncPtr (Params (Type void)) (Struct point))))                    
                ",
            ),
        ];

        run_tests(Parser::parse_program, test_cases);
    }

    // TODO: Add nested enums
    #[test]
    fn struct_nested() {
        let test_cases = vec![
            (
                "struct A {  struct B { int x; } b;  }; ",
                "
                (Struct A (Members
                    (Struct B (Members
                        (Member (Name x (Type int)))
                    ))
                    (Member (Name b (Struct B)))
                ))
                    
                ",
            ),
            (
                "
                struct A {
                    struct B {
                        struct C { int x; } c;
                    } b;
                };
                ",
                "
                (Struct A (Members
                    (Struct B (Members
                        (Struct C (Members
                            (Member (Name x (Type int)))
                        ))
                        (Member (Name c (Struct C)))
                      ))
                    (Member (Name b (Struct B)))
                ))

                ",
            ),
            (
                "
                struct A {
                    struct { int x; } b;
                };
                ",
                "
                (Struct A (Members
                    (Struct Anon-TagType-0 (Members
                        (Member (Name x (Type int)))
                    ))
                    (Member (Name b (Struct Anon-TagType-0)))
                ))
                ",
            ),
            (
                "
                struct Outer {
                    struct Inner { int x; } i;
                    struct Another { int y; } a;
                };
                ",
                "
                (Struct Outer (Members
                    (Struct Inner (Members
                        (Member (Name x (Type int)))
                    ))
                    (Member (Name i (Struct Inner)))
                    (Struct Another (Members
                        (Member (Name y (Type int)))
                    ))
                    (Member (Name a (Struct Another)))
                ))
                ",
            ),
            (
                "
                struct A {
                    struct B *ptr;
                    struct B { int x; } b;
                };
                ",
                "
                (Struct A (Members
                    (Member (Name ptr (Ptr (Struct B))))
                    (Struct B (Members
                        (Member (Name x (Type int)))
                    ))
                    (Member (Name b (Struct B)))
                ))
                ",
            ),
            (
                "
                struct A {
                    struct { int x; } grid[3][4];
                };
                ",
                "
                (Struct A (Members
                    (Struct Anon-TagType-0 (Members
                        (Member (Name x (Type int)))
                    ))
                    (Member (Name grid (Arr (Num 4) (Arr (Num 3) (Struct Anon-TagType-0)))))
                ))
                ",
            ),
            (
                "struct A{ struct B; int x; };",
                "
                (Struct A (Members
                    (Struct B)
                    (Member (Name x (Type int)))
                ))
                ",
            ),
        ];

        run_tests(Parser::parse_program, test_cases);
    }
}
