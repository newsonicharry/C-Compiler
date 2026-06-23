use crate::lexer::language_features::AssignmentTypes;
use crate::lexer::language_features::OperatorTypes;
use crate::lexer::language_features::{KeywordTypes, LiteralTypes};
use crate::lexer::lexer::{Lexer, TokenTypes};
use crate::parser::expression_parser::parse_expression;
use crate::parser::parser::parse_statement;
use crate::parser::parser::{GlobalNode, StatementNode};
use crate::parser::tag_types::helper::is_tag_type_keyword;
use crate::parser::tag_types::helper::{
    TagKeywordUsage, parse_tag_type_qualifiers, tag_type_keyword_usage,
};
use crate::parser::type_parser::{TypeNode, parse_type};

pub struct Struct {
    pub is_defined: bool,
    pub name: Option<String>,
    pub members: Vec<StructMember>,
}

impl Struct {
    pub fn display(&self, indentation: usize) -> String {
        let mut output = String::new();

        let indent_str = " ".repeat(indentation);

        output.push_str(&format!("{indent_str}(Struct"));

        if let Some(name) = self.name.clone() {
            output.push_str(&format!(" {name}"));
        }

        if self.is_defined {
            output.push_str(&format!(" (Members"));

            for member in &self.members {
                output.push_str(&format!("\n{}", member.display(indentation + 2)));
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

pub enum StructMember {
    NormalType {
        item_type: TypeNode,
        bit_field: Option<u64>,
    },
    DefinedStruct {
        defined: Struct,
    },
}

impl StructMember {
    pub fn display(&self, indentation: usize) -> String {
        let mut output = String::new();

        let indent_str = " ".repeat(indentation);
        if let StructMember::NormalType {
            item_type,
            bit_field,
        } = self
        {
            output += &format!("{indent_str}(Member {}", item_type);

            if let Some(bitfield) = bit_field {
                output.push_str(&format!("(Bitfield {})", bitfield.to_string()));
            }
        }

        if let StructMember::DefinedStruct { defined } = self {
            output += &format!("{}", defined.display(indentation));
        }

        output.push_str(")");

        output
    }
}

/// Parses anything that uses a struct keyword
/// This includes struct definitions, declarations and variables
pub fn parse_struct_keyword(lexer: &mut Lexer) -> Result<Vec<GlobalNode>, String> {
    let usage = tag_type_keyword_usage(lexer)?;

    if matches!(usage, TagKeywordUsage::Variable) {
        let vars = parse_statement(lexer)?;

        let vars = vars
            .iter()
            .map(|x| GlobalNode::Variable {
                expr_statement: x.clone(),
            })
            .collect::<Vec<GlobalNode>>();

        return Ok(vars);
    }

    if matches!(usage, TagKeywordUsage::Definition) {
        let mut struct_and_vars = Vec::new();

        let mut struct_qualifiers = parse_tag_type_qualifiers(lexer)?;

        let defined_struct = parse_struct_definition(lexer)?;

        let struct_name = defined_struct.name.clone();

        struct_and_vars.push(GlobalNode::Struct(defined_struct));

        struct_qualifiers.extend(parse_tag_type_qualifiers(lexer)?);

        let struct_type = TypeNode::Struct {
            name: struct_name,
            qualifiers: struct_qualifiers,
        };

        let defined_vars: Vec<GlobalNode> = parse_vars_after_type::<true>(lexer, &struct_type)?
            .iter()
            .map(|x| GlobalNode::Variable {
                expr_statement: x.clone(),
            })
            .collect();

        struct_and_vars.extend(defined_vars);

        return Ok(struct_and_vars);
    }

    // if its a declaration
    if matches!(usage, TagKeywordUsage::Declaration) {
        return Ok(vec![GlobalNode::Struct(parse_struct_declaration(lexer)?)]);
    }

    unreachable!()
}

fn parse_struct_member(lexer: &mut Lexer) -> Result<Vec<StructMember>, String> {
    if is_tag_type_keyword(lexer, &KeywordTypes::Struct)? {
        let struct_items = parse_struct_keyword(lexer)?;
        let mut all_members = Vec::new();

        for item in struct_items {
            let struct_member = match item {
                GlobalNode::Struct(data) => StructMember::DefinedStruct { defined: data },

                GlobalNode::Variable { expr_statement } => {
                    let StatementNode::Expression { var_type, .. } = expr_statement else {
                        unreachable!()
                    };

                    StructMember::NormalType {
                        item_type: var_type,
                        bit_field: None,
                    }
                }
                _ => todo!(),
            };

            all_members.push(struct_member);
        }

        return Ok(all_members);
    }

    let member = parse_type(lexer)?;

    let Some(next_token) = lexer.peek() else {
        return Err(String::from(
            "Expected either a semicolon or colon at the end of struct member, got nothing",
        ));
    };

    let mut bit_field = None;

    if matches!(next_token, TokenTypes::Operator(OperatorTypes::Colon)) {
        lexer.advance();

        bit_field = Some(lexer.expect_extract(|x| match x {
            TokenTypes::Literal(LiteralTypes::Integer(integer)) => Some(integer.value as u64),
            _ => None,
        })?);
    }

    lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

    let final_member = StructMember::NormalType {
        item_type: member,
        bit_field,
    };

    Ok(vec![final_member])
}

/// Parses struct declarations
/// (e.g. Struct Point; )
fn parse_struct_declaration(lexer: &mut Lexer) -> Result<Struct, String> {
    // qualifiers in a declaration are not used
    parse_tag_type_qualifiers(lexer)?;

    // move past the struct keyword
    lexer.advance();

    let Some(TokenTypes::Identifier(struct_name)) = lexer.peek() else {
        unreachable!()
    };

    lexer.advance();

    parse_tag_type_qualifiers(lexer)?;

    let declared_struct = Struct {
        is_defined: false,
        name: Some(struct_name),
        members: vec![],
    };

    lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

    Ok(declared_struct)
}

/// Parses struct definitions
/// These include stuct with a left and right curly brace that may or may not include members
/// This does not account for variables defined subsequently with the struct
fn parse_struct_definition(lexer: &mut Lexer) -> Result<Struct, String> {
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
        let final_member = parse_struct_member(lexer)?;
        members.extend(final_member);
    }

    lexer.advance();

    let final_struct = Struct {
        is_defined: true,
        name: name.clone(),
        members,
    };

    Ok(final_struct)
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
) -> Result<Vec<StatementNode>, String> {
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

            final_var = StatementNode::Expression {
                var_type: var_type.clone(),
                r_value: Some(parse_expression(lexer, 3)?),
            };
        }
        // its a declaration
        else {
            final_var = StatementNode::Expression {
                var_type: var_type.clone(),
                r_value: None,
            };
        }

        all_vars.push(final_var);
    }

    lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

    Ok(all_vars)
}

#[cfg(test)]
mod tests {
    use crate::parser::helper::run_tests;
    use crate::parser::parser::parse_program;

    #[test]
    fn struct_creation() {
        let test_cases = vec![
            ("struct Point;", "(Struct Point)"),
            ("const struct Point;", "(Struct Point)"),
            ("struct Point const;", "(Struct Point)"),
            ("struct {};", "(Struct (Members))"),
            ("struct Point{};", "(Struct Point (Members))"),
            (
                "struct Point{int x; int y;};",
                "(Struct Point (Members
                  (Member (Name x (Type int)))
                  (Member (Name y (Type int)))
                ))",
            ),
            (
                "struct {int x; int y;};",
                "(Struct (Members
                  (Member (Name x (Type int)))
                  (Member (Name y (Type int)))
                ))",
            ),
            (
                "struct Point{int x : 3; int y : 2;};",
                "(Struct Point (Members
                  (Member (Name x (Type int)) (Bitfield 3))
                  (Member (Name y (Type int)) (Bitfield 2))
                ))",
            ),
        ];

        run_tests(parse_program, test_cases);
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

        run_tests(parse_program, test_cases);
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

        run_tests(parse_program, test_cases);
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

        run_tests(parse_program, test_cases);
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

        run_tests(parse_program, test_cases);
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
                    )))
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
                        )))
                        (Member (Name c (Struct C)))
                      )))
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
                    (Struct (Members
                        (Member (Name x (Type int)))
                    )))
                    (Member (Name b (Struct)))
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
                    )))
                    (Member (Name i (Struct Inner)))
                    (Struct Another (Members
                        (Member (Name y (Type int)))
                    )))
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
                    )))
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
                    (Struct (Members
                        (Member (Name x (Type int)))
                    )))
                    (Member (Name grid (Arr (Num 4) (Arr (Num 3) (Struct)))))
                ))  
                ",
            ),
            (
                "struct A{ struct B; int x; };",
                "
                (Struct A (Members
                    (Struct B))
                    (Member (Name x (Type int)))
                ))
                ",
            ),
        ];

        run_tests(parse_program, test_cases);
    }
}
