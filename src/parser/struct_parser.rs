use crate::lexer::language_features::AssignmentTypes;
use crate::lexer::lexer::{Lexer, TokenTypes};
use crate::parser::expression_parser::ExprNode;
use crate::parser::helper::verify_next_in_comma_list;
use crate::parser::parser::{GlobalNode, StatementNode, parse_var};
use crate::parser::type_parser::{TypeNode, is_valid_var_name};

pub struct Struct {
    pub is_defined: bool,
    pub name: Option<String>,
    pub members: Vec<TypeNode>,
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

    if matches!(lexer.peek(), Some(TokenTypes::Semicolon)) {
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
        let member = parse_var(lexer)?;

        let StatementNode::Expression { var_type, r_value } = member else {
            unreachable!()
        };

        if r_value.is_some() {
            return Err(String::from(
                "Struct member must be a declaration, definition given",
            ));
        }
        members.push(var_type);
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
        let assigned_to = parse_aggregate_init(lexer, struct_name)?;

        let final_var = StatementNode::Expression {
            var_type: final_type,
            r_value: Some((AssignmentTypes::SimpleAssignment, assigned_to)),
        };

        return Ok(final_var);
    }

    todo!()
}

fn parse_aggregate_init(
    lexer: &mut Lexer,
    struct_name: &Option<String>,
) -> Result<ExprNode, String> {
    todo!()
}
