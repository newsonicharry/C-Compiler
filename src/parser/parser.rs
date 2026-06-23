use crate::lexer::language_features::KeywordTypes;
use crate::lexer::language_features::{AssignmentTypes, OperatorTypes};
use crate::lexer::lexer::{Lexer, TokenTypes};
use crate::parser::aggregate_init::{AggregateInit, parse_aggregate_init};
use crate::parser::expression_parser::{ExprNode, parse_expression};
use crate::parser::tag_types::enum_parser::Enum;
use crate::parser::tag_types::helper::is_tag_type_keyword;
use crate::parser::tag_types::struct_parser::{
    Struct, parse_struct_keyword, parse_vars_after_type,
};
use crate::parser::type_parser::{TypeNode, parse_parameter_list, parse_type};
use crate::parser::typedef::is_typedef;
use std::fmt::Display;

pub struct Root(Vec<GlobalNode>);

impl Display for Root {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();

        for node in self.0.iter() {
            output.push_str(&format!("{node}\n"));
        }

        if output.chars().last() == Some('\n') {
            output.pop();
        }

        write!(display, "{output}")
    }
}

pub enum GlobalNode {
    // functions, variables, struct, union, enum, typedef
    Function {
        name: String,
        return_type: TypeNode,
        params: Vec<TypeNode>,
        body: Option<StatementNode>,
    },

    Variable {
        expr_statement: StatementNode,
    },

    Union {
        name: Option<String>,
        members: Vec<TypeNode>,
    },

    Struct(Struct),
    Enum(Enum),

    Typedef {}, // Todo, don't want to even touch these yet
}

impl Display for GlobalNode {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let final_str = self.display(0);

        write!(display, "{final_str}")
    }
}

impl GlobalNode {
    fn display(&self, indentation: usize) -> String {
        let mut output = String::new();

        let indent_chars = " ".repeat(indentation + 2);

        match self {
            Self::Function {
                name,
                return_type,
                params,
                body,
            } => {
                output.push_str(&format!("(Func {return_type} {name}"));

                if !params.is_empty() {
                    output.push_str(" (Params ");
                    for param in params {
                        output.push_str(&param.to_string());
                    }
                    output.push_str(")");
                }

                if let Some(body) = body {
                    output.push_str(&format!("\n{body}\n"));
                }

                output.push_str(")");
            }

            Self::Variable { expr_statement } => {
                output.push_str(&expr_statement.to_string());
            }

            Self::Struct(data) => {
                output.push_str(&data.display(indentation));
            }

            _ => todo!(),
        }

        output
    }
}

#[derive(Clone)]
pub enum StatementNode {
    // block, expression, if, switch, while, do, for, return, break, continue, goto, label, case, default
    Block {
        statements: Vec<StatementNode>,
    },

    Expression {
        var_type: TypeNode,
        r_value: Option<ExprNode>,
    },
}

#[derive(Clone, Debug)]
pub struct InitalizerNode {
    pub aggregate: Option<AggregateInit>,
    pub expr: Option<ExprNode>,
}

impl Display for InitalizerNode {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let final_str;

        if let Some(aggregate) = self.aggregate.clone() {
            final_str = aggregate.to_string();
        } else {
            let Some(expr) = self.expr.clone() else {
                unreachable!()
            };

            final_str = expr.to_string();
        }

        write!(display, "{final_str}")
    }
}

impl Display for StatementNode {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let final_str = Self::display(self);

        write!(display, "{final_str}")
    }
}

impl StatementNode {
    fn display(&self) -> String {
        let mut output = String::new();

        match self {
            Self::Expression { var_type, r_value } => {
                output.push_str(&format!("(Variable {var_type}"));

                if let Some(expression) = r_value {
                    output.push_str(&format!(" {expression}",));
                }

                output.push_str(")");
            }

            Self::Block { statements } => {
                for statement in statements {
                    output.push_str(&format!("\n{statement}\n"));
                }
            }

            _ => todo!(),
        }

        output
    }
}

pub fn parse_program(lexer: &mut Lexer) -> Result<Root, String> {
    let mut root = Root(Vec::new());

    while let Some(token) = lexer.peek() {
        if is_typedef(lexer) {
            println!("is typedef");
            // do something
            continue;
        }

        match token {
            TokenTypes::DataType(_) => match is_tag_type_keyword(lexer, &KeywordTypes::Struct)? {
                true => root.0.extend(parse_struct_keyword(lexer)?),
                false => root.0.push(parse_function_or_var(lexer)?),
            },

            TokenTypes::Keyword(KeywordTypes::Struct) => {
                root.0.extend(parse_struct_keyword(lexer)?);
            }

            TokenTypes::Semicolon => {
                lexer.advance();
            }

            _ => unimplemented!(),
        }
    }

    Ok(root)
}

fn parse_function_or_var(lexer: &mut Lexer) -> Result<GlobalNode, String> {
    let mut is_function = false;
    lexer.set_flag();

    let mut found_identifier = false;
    while let Some(token) = lexer.next()
        && !matches!(token, TokenTypes::Semicolon)
        && !matches!(token, TokenTypes::Assignment(_))
    {
        if matches!(token, TokenTypes::Operator(OperatorTypes::LParen)) {
            // if we already found a parenthesiss if we found the variable already it must be a function
            // otherwise its just a type with parenthesis around its variable name
            is_function = found_identifier;
            break;
        }

        if matches!(token, TokenTypes::Identifier(_)) {
            found_identifier = true;
        }
    }

    lexer.recede_to_flag();

    match is_function {
        true => parse_function(lexer),
        false => todo!(), // false => Ok(GlobalNode::Variable {
                          //     expr_statement: parse_statement(lexer)?,
                          // }),
    }
}

/// A high level variable parser
/// Does not support struct parsing
pub fn parse_statement(lexer: &mut Lexer) -> Result<Vec<StatementNode>, String> {
    let mut var_type = parse_type(lexer)?;

    let next_token = lexer.force_peek("Expected end of var, got nothing")?;

    if matches!(next_token, TokenTypes::Semicolon) {
        lexer.advance();

        let final_var = StatementNode::Expression {
            var_type: var_type.clone(),
            r_value: None,
        };

        return Ok(vec![final_var]);
    }

    let mut all_vars = vec![];

    if matches!(
        next_token,
        TokenTypes::Assignment(AssignmentTypes::SimpleAssignment)
    ) {
        lexer.advance();

        let first_var = StatementNode::Expression {
            var_type: var_type.clone(),
            r_value: Some(parse_expression(lexer, 3)?),
        };

        all_vars.push(first_var);
    } else if matches!(next_token, TokenTypes::Operator(OperatorTypes::Comma)) {
        let first_var = StatementNode::Expression {
            var_type: var_type.clone(),
            r_value: None,
        };

        all_vars.push(first_var);
    }

    let additional_vars = parse_vars_after_type::<false>(lexer, &var_type.get_most_nested_layer())?;
    all_vars.extend(additional_vars);

    Ok(all_vars)
}

fn parse_function(lexer: &mut Lexer) -> Result<GlobalNode, String> {
    let return_type = parse_type(lexer)?;

    let function_name = lexer.expect_extract(|x| match x {
        TokenTypes::Identifier(identifier) => Some(identifier),
        _ => None,
    })?;

    lexer.expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::LParen)))?;

    let param_list = parse_parameter_list(lexer)?;

    let final_function;

    let Some(next_token) = lexer.peek() else {
        return Err(String::from("Expected end of function, got nothing"));
    };

    match next_token {
        TokenTypes::Semicolon => {
            final_function = GlobalNode::Function {
                name: function_name,
                return_type,
                params: param_list,
                body: None,
            }
        }

        TokenTypes::LCurlyBrace => {
            final_function = GlobalNode::Function {
                name: function_name,
                return_type,
                params: param_list,
                body: Some(parse_block(lexer)?),
            }
        }

        _ => {
            return Err(String::from(format!(
                "Got unexpected token {:?}",
                next_token
            )));
        }
    }

    lexer.advance();

    Ok(final_function)
}

fn parse_block(lexer: &mut Lexer) -> Result<StatementNode, String> {
    todo!()
}

pub const STOP_AT_COMMA: bool = false;

pub fn parse_initalizer<const SHOULD_PARSE_COMMA: bool>(
    lexer: &mut Lexer,
) -> Result<InitalizerNode, String> {
    let token = lexer.force_peek("Expected initalizer, got nothing")?;

    let start_precedence = match SHOULD_PARSE_COMMA {
        true => 0,
        false => 3,
    };

    let mut aggregate_node = None;
    let mut expr_node = None;

    match token {
        TokenTypes::LCurlyBrace => {
            aggregate_node = Some(parse_aggregate_init(lexer)?);
        }

        TokenTypes::Literal(_)
        | TokenTypes::Identifier(_)
        | TokenTypes::Keyword(KeywordTypes::Sizeof) => {
            expr_node = Some(parse_expression(lexer, start_precedence)?);
        }

        TokenTypes::Operator(op) if op.potential_unary() => {
            expr_node = Some(parse_expression(lexer, start_precedence)?);
        }

        _ => {
            return Err(format!(
                "Unexpected token of type {token} to start initalizer"
            ));
        }
    }

    Ok(InitalizerNode {
        aggregate: aggregate_node,
        expr: expr_node,
    })
}
