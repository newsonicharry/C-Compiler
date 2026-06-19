use crate::lexer::language_features::KeywordTypes;
use crate::lexer::language_features::{AssignmentTypes, OperatorTypes};
use crate::lexer::lexer::{Lexer, TokenTypes};
use crate::parser::aggregate_init::{AggregateInit, parse_aggregate_init};
use crate::parser::expression_parser::{ExprNode, parse_expression};
use crate::parser::struct_parser::{Struct, parse_struct_keyword};
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

pub struct EnumMember {
    name: String,
    value: Option<i32>,
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

    Struct {
        data: Struct,
    },

    Enum {
        name: Option<String>,
        members: Vec<EnumMember>,
    },

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

            Self::Struct { data } => {
                output.push_str(&format!("(Struct"));

                if let Some(name) = data.name.clone() {
                    output.push_str(&format!(" {name}"));
                }

                if data.is_defined {
                    output.push_str(&format!(" (Members"));

                    for member in &data.members {
                        output.push_str(&format!("\n{indent_chars}{member}"));
                    }

                    if !data.members.is_empty() {
                        output.push_str("\n");
                    }
                    output.push_str(")");
                }

                output.push_str(")");
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
        r_value: Option<(AssignmentTypes, InitalizerNode)>,
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
                todo!()
                // output.push_str(&format!("(Variable {var_type}"));

                // if let Some((assign_op, expression)) = r_value {
                //     output.push_str(&format!(
                //         " {assign_op} {}",
                //         expression
                //             .to_string()
                //             .chars()
                //             .filter(|x| *x != '\n')
                //             .collect::<String>(),
                //     ));
                // }

                // output.push_str(")");
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
            TokenTypes::DataType(_) => {
                root.0.push(parse_function_or_var(lexer)?);
            }

            TokenTypes::Keyword(KeywordTypes::Struct) => {
                root.0.extend(parse_struct_keyword(lexer)?);
            }

            TokenTypes::Semicolon => {} // semicolons can be by themselves
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
        false => Ok(GlobalNode::Variable {
            expr_statement: parse_var(lexer)?,
        }),
    }
}

pub fn parse_var(lexer: &mut Lexer) -> Result<StatementNode, String> {
    let var_type = parse_type(lexer)?;

    let Some(next_token) = lexer.peek() else {
        return Err(String::from("Expected end of var, got nothing"));
    };

    if matches!(next_token, TokenTypes::Semicolon) {
        lexer.advance();

        let final_var = StatementNode::Expression {
            var_type,
            r_value: None,
        };

        return Ok(final_var);
    }

    let assign_op = lexer.expect_extract(|x| match x {
        TokenTypes::Assignment(assign_op) => Some(assign_op),
        _ => None,
    })?;

    let initalizer = parse_initalizer(lexer)?;

    lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

    let final_var = StatementNode::Expression {
        var_type,
        r_value: Some((assign_op, initalizer)),
    };

    Ok(final_var)
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

pub fn parse_initalizer(lexer: &mut Lexer) -> Result<InitalizerNode, String> {
    let token = lexer.force_peek("Expected initalizer, got nothing")?;

    let mut aggregate_node = None;
    let mut expr_node = None;

    match token {
        TokenTypes::LCurlyBrace => {
            aggregate_node = Some(parse_aggregate_init(lexer)?);
        }

        TokenTypes::Literal(_)
        | TokenTypes::Identifier(_)
        | TokenTypes::Keyword(KeywordTypes::Sizeof) => {
            expr_node = Some(parse_expression(lexer, 0)?);
        }

        TokenTypes::Operator(op) if op.potential_unary() => {
            expr_node = Some(parse_expression(lexer, 0)?);
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
