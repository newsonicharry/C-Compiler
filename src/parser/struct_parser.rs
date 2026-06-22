use crate::lexer::language_features::AssignmentTypes;
use crate::lexer::language_features::OperatorTypes;
use crate::lexer::language_features::{KeywordTypes, LiteralTypes};
use crate::lexer::lexer::{Lexer, TokenTypes};
use crate::parser::expression_parser::parse_expression;
use crate::parser::helper::verify_next_in_comma_list;
use crate::parser::parser::{GlobalNode, StatementNode};
use crate::parser::type_parser::{TypeNode, is_valid_var_name, parse_type};
use std::fmt::Display;

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

enum StructMember {
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

#[derive(Debug)]
enum StructKeywordUsage {
    Definition,
    Declaration,
    Variable,
}

fn struct_keyword_usage(lexer: &mut Lexer) -> Result<StructKeywordUsage, String> {
    lexer.set_flag();

    lexer.advance(); // move past the struct keyword

    // Move past the struct name if it exists
    if let Some(TokenTypes::Identifier(_)) = lexer.peek() {
        lexer.next();
    }

    let next_token = lexer.force_peek("Expected next token in struct definition, got nothing")?;

    // make sure we don't mess up the parsing for our parsing functions
    lexer.recede_to_flag();

    // if its a variable
    // includes left parenthesis and start because it could be a function pointer or pointer
    if matches!(next_token, TokenTypes::Identifier(_))
        || matches!(next_token, TokenTypes::Operator(OperatorTypes::LParen))
        || matches!(next_token, TokenTypes::Operator(OperatorTypes::Star))
    {
        return Ok(StructKeywordUsage::Variable);
    }

    if matches!(next_token, TokenTypes::LCurlyBrace) {
        return Ok(StructKeywordUsage::Definition);
    }

    if matches!(next_token, TokenTypes::Semicolon) {
        return Ok(StructKeywordUsage::Declaration);
    }

    Err(String::from(&format!(
        "Unexpected next token {next_token}, expected struct variable, struct definition or struct declaration",
    )))
}

// struct definition / declaration
// struct definition and objects
// variable definition/definition of type struct
pub fn parse_struct_keyword(lexer: &mut Lexer) -> Result<Vec<GlobalNode>, String> {
    let usage = struct_keyword_usage(lexer)?;
    println!("{:?}", usage);

    if matches!(usage, StructKeywordUsage::Variable) {
        let vars = parse_struct_var(lexer)?;

        let vars = vars
            .iter()
            .map(|x| GlobalNode::Variable {
                expr_statement: x.clone(),
            })
            .collect::<Vec<GlobalNode>>();

        return Ok(vars);
    }

    if matches!(usage, StructKeywordUsage::Definition) {
        let mut struct_and_vars = Vec::new();

        let defined_struct = parse_struct_definition(lexer)?;
        let struct_name = defined_struct.name.clone();

        struct_and_vars.push(GlobalNode::Struct {
            data: defined_struct,
        });

        if matches!(lexer.peek(), Some(TokenTypes::Identifier(_))) {
            // struct_and_vars.extend(
            //     parse_vars_from_struct(lexer, &struct_name)?
            //         .iter()
            //         .map(|x| GlobalNode::Variable {
            //             expr_statement: x.clone(),
            //         }),
            // );
        }

        return Ok(struct_and_vars);
    }

    // if its a declaration
    if matches!(usage, StructKeywordUsage::Declaration) {
        return Ok(vec![GlobalNode::Struct {
            data: parse_struct_declaration(lexer),
        }]);
    }

    unreachable!()
}

fn parse_struct_member(lexer: &mut Lexer) -> Result<StructMember, String> {
    if matches!(
        lexer.peek(),
        Some(TokenTypes::Keyword(KeywordTypes::Struct))
    ) {
        let usage = struct_keyword_usage(lexer)?;

        match usage {
            StructKeywordUsage::Definition => {
                return Ok(StructMember::DefinedStruct {
                    defined: parse_struct_definition(lexer)?,
                });
            }

            StructKeywordUsage::Declaration => {
                return Ok(StructMember::DefinedStruct {
                    defined: parse_struct_declaration(lexer),
                });
            }

            StructKeywordUsage::Variable => {}
        }
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

    Ok(final_member)
}

fn parse_struct_declaration(lexer: &mut Lexer) -> Struct {
    lexer.advance();

    let Some(TokenTypes::Identifier(struct_name)) = lexer.peek() else {
        unreachable!()
    };

    lexer.advance();

    let declared_struct = Struct {
        is_defined: false,
        name: Some(struct_name),
        members: vec![],
    };

    declared_struct
}

// definition parsing
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
        members.push(final_member);
    }

    lexer.advance();

    let final_struct = Struct {
        is_defined: true,
        name: name.clone(),
        members,
    };

    Ok(final_struct)
}

// variable parsing

fn parse_struct_var(
    lexer: &mut Lexer,
    // struct_name: &Option<String>,
) -> Result<Vec<StatementNode>, String> {
    // fn update_var(final_type: &mut TypeNode, lexer: &mut Lexer) -> Result<(), String> {
    //     let var_name = lexer.expect_extract(|x| match x {
    //         TokenTypes::Identifier(var_name) => Some(var_name),
    //         _ => None,
    //     })?;
    //     final_type.change_var_name(&var_name)?;
    //     Ok(())
    // }

    // let mut final_type;

    // match struct_name {
    //     Some(name) => {
    //         final_type = TypeNode::Variable {
    //             name: String::new(),
    //             held_value: TypeNode::Struct {
    //                 name,
    //                 qualifiers: vec![],
    //             },
    //         };
    //     }

    //     None => final_type = parse_type(lexer)?,
    // }

    let mut final_type = parse_type(lexer)?;

    final_type.error_if_not_variable()?;

    let mut all_vars = Vec::new();

    loop {
        let Some(next_token) = lexer.peek() else {
            return Err(String::from(
                "Expected end of struct variable declaration, got nothing",
            ));
        };

        // early exit if the last var is a declaration
        if next_token == TokenTypes::Semicolon {
            let final_var = StatementNode::Expression {
                var_type: final_type.clone(),
                r_value: None,
            };

            all_vars.push(final_var);
            break;
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
                var_type: final_type.clone(),
                r_value: Some(parse_expression(lexer, 3)?),
            };
        }
        // its a declaration
        else {
            final_var = StatementNode::Expression {
                var_type: final_type.clone(),
                r_value: None,
            };
        }

        all_vars.push(final_var);

        let next_token = lexer.force_peek("Expected end of variable definition, got nothing")?;

        // Could be another variable assigned after the original one
        if matches!(next_token, TokenTypes::Operator(OperatorTypes::Comma)) {
            lexer.advance();
            let var_name = lexer.expect_extract(|x| match x {
                TokenTypes::Identifier(var_name) => Some(var_name),
                _ => None,
            })?;
            final_type.change_var_name(&var_name)?;
            continue;
        }

        if matches!(next_token, TokenTypes::Semicolon) {
            break;
        }

        // Variable assignment can only end wiht a comma or semi colon
        return Err(format!(
            "Expected comma or semicolon after variable definition, got token of type {next_token}"
        ));
    }

    lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

    // todo!()
    Ok(all_vars)
}

/*
Everything that I need to add

#Structs in structs
struct Point{
  struct AA{
    int z;
    struct AAA{
      int z;
    } zz;
  } another;

};

*/

#[cfg(test)]
mod tests {
    use crate::parser::helper::run_tests;
    use crate::parser::parser::parse_program;

    #[test]
    fn struct_creation() {
        let test_cases = vec![
            ("struct Point;", "(Struct Point)"),
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
}
