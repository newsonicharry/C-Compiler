use crate::lexer::language_features::AssignmentTypes;
use crate::lexer::language_features::LiteralTypes;
use crate::lexer::language_features::OperatorTypes;
use crate::lexer::lexer::{Lexer, TokenTypes};
use crate::parser::helper::verify_next_in_comma_list;
use crate::parser::parser::{GlobalNode, StatementNode};
use crate::parser::type_parser::{TypeNode, is_valid_var_name, parse_type};
use std::fmt::Display;

pub struct Struct {
    pub is_defined: bool,
    pub name: Option<String>,
    pub members: Vec<StructMember>,
}

pub struct StructMember {
    pub item_type: TypeNode,
    pub bit_field: Option<u64>,
}

impl Display for StructMember {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::from(&format!("(Member {}", self.item_type));

        if let Some(bitfield) = self.bit_field {
            output.push_str(&format!("( Bitfield {})", bitfield.to_string()));
        }

        output.push_str(")");

        write!(display, "{output}")
    }
}

// struct definition / declaration
// struct definition and objects
// variable definition/definition of type struct
pub fn parse_struct_keyword(lexer: &mut Lexer) -> Result<Vec<GlobalNode>, String> {
    lexer.advance(); // move past the "struct"

    let mut struct_name = None;

    if let Some(TokenTypes::Identifier(name)) = lexer.peek() {
        struct_name = Some(name);
        lexer.next();
    }

    // if its a variable
    if matches!(lexer.peek(), Some(TokenTypes::Identifier(_))) {
        let final_var = parse_struct_var(lexer, &struct_name)?;

        let final_var = GlobalNode::Variable {
            expr_statement: final_var,
        };

        return Ok(vec![final_var]);
    }

    // if its a definition (could have objects defined after it)
    if matches!(lexer.peek(), Some(TokenTypes::LCurlyBrace)) {
        let mut struct_and_vars = Vec::new();

        let defined_struct = parse_struct_definition(lexer, &struct_name)?;

        struct_and_vars.push(GlobalNode::Struct {
            data: defined_struct,
        });

        if matches!(lexer.peek(), Some(TokenTypes::Identifier(_))) {
            struct_and_vars.extend(
                parse_vars_from_struct(lexer, &struct_name)?
                    .iter()
                    .map(|x| GlobalNode::Variable {
                        expr_statement: x.clone(),
                    }),
            );
        }

        lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

        return Ok(struct_and_vars);
    }

    // if its a declaration
    if matches!(lexer.peek(), Some(TokenTypes::Semicolon)) {
        lexer.advance();
        let declared_struct = Struct {
            is_defined: false,
            name: struct_name,
            members: vec![],
        };

        return Ok(vec![GlobalNode::Struct {
            data: declared_struct,
        }]);
    }

    Err(String::from(&format!(
        "Unexpected next token {:?}, expected ",
        lexer.peek()
    )))
}

// definition parsing
fn parse_struct_definition(lexer: &mut Lexer, name: &Option<String>) -> Result<Struct, String> {
    let mut members = Vec::new();

    lexer.advance(); // move past the left curly brace

    while !matches!(lexer.peek(), Some(TokenTypes::RCurlyBrace)) {
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
                TokenTypes::Literal(LiteralTypes::Integer(integer)) => Some(integer),
                _ => None,
            })?);
        }

        lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

        let final_member = StructMember {
            item_type: member,
            bit_field,
        };

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

// check for defined objects in the struct
fn parse_vars_from_struct(
    lexer: &mut Lexer,
    struct_name: &Option<String>,
) -> Result<Vec<StatementNode>, String> {
    match lexer.peek() {
        Some(TokenTypes::Identifier(_)) => {}

        Some(TokenTypes::Semicolon) => {
            return Ok(vec![]);
        }

        _ => {
            return Err(String::from(
                "Struct definition must end in a semicolon or define a variable",
            ));
        }
    }

    let mut defined_vars = Vec::new();

    while !matches!(lexer.peek(), Some(TokenTypes::Semicolon)) {
        if matches!(lexer.peek(), Some(TokenTypes::Comma)) {
            return Err(String::from("Unexpected comma after struct definition"));
        }

        let var_name = lexer.expect_extract(|x| match x {
            TokenTypes::Identifier(var) => Some(var),
            _ => None,
        })?;

        if !is_valid_var_name(&var_name) {
            return Err(String::from("Variable does not have a valid variable name"));
        }

        let variable_type = TypeNode::Variable {
            name: var_name,
            held_value: Box::new(TypeNode::Struct {
                name: struct_name.clone(),
                qualifiers: vec![],
            }),
        };

        defined_vars.push(StatementNode::Expression {
            var_type: variable_type,
            r_value: None,
        });

        verify_next_in_comma_list(
            lexer,
            TokenTypes::Semicolon,
            "Unexpected end to variable definitions after struct definition",
        )?;
    }

    Ok(defined_vars)
}

// variable parsing

fn parse_struct_var(
    lexer: &mut Lexer,
    struct_name: &Option<String>,
) -> Result<StatementNode, String> {
    let variable_name = lexer.expect_extract(|x| match x {
        TokenTypes::Identifier(var_name) => Some(var_name),
        _ => None,
    })?;

    let Some(next_token) = lexer.peek() else {
        return Err(String::from(
            "Expected end of struct variable declaration, got nothing",
        ));
    };

    let final_type = TypeNode::Variable {
        name: variable_name,
        held_value: Box::new(TypeNode::Struct {
            name: struct_name.clone(),
            qualifiers: vec![],
        }),
    };

    if next_token == TokenTypes::Semicolon {
        let final_var = StatementNode::Expression {
            var_type: final_type,
            r_value: None,
        };

        return Ok(final_var);
    }

    // should be an assignment
    lexer.expect(|x| matches!(x, TokenTypes::Assignment(AssignmentTypes::SimpleAssignment)))?;

    let Some(next_token) = lexer.peek() else {
        return Err(String::from(
            "Expected struct variable definition after assingment, got nothing",
        ));
    };

    if next_token == TokenTypes::LCurlyBrace {
        todo!()
        // let assigned_to = parse_aggregate_init(lexer)?;

        // let final_var = StatementNode::Expression {
        //     var_type: final_type,
        //     r_value: Some((AssignmentTypes::SimpleAssignment, assigned_to)),
        // };

        // return Ok(final_var);
    }

    todo!()
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

#Initialization
struct Point p = {1, 2};
struct Point p = {.x = 1, .y = 2};
struct Point p = (struct Point){1, 2};

# Additional variables in the same declaration
struct Point p, q;
struct Point p = {1,2}, q = {3,4};
int x = 10, y = 20;

#Arrays
struct Point p[] = {{1,2}, {3,4}};

#Function declarations
struct Point p(void);
struct Point p(int);
struct Point p(int, char *);

#Typedef declarations
typedef struct Point p;

#Struct member access or assignment
p = q;
p.x = 1;

*/

#[cfg(test)]
mod tests {
    use crate::parser::helper::run_tests;
    use crate::parser::parser::parse_program;

    #[test]
    fn test_struct_creation() {
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
}
