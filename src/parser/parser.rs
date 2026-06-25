use crate::lexer::language_features::KeywordTypes;
use crate::lexer::language_features::{AssignmentTypes, OperatorTypes};
use crate::lexer::lexer::{Lexer, TokenTypes};
use crate::parser::aggregate_init::{AggregateInit, parse_aggregate_init};
use crate::parser::expression_parser::{ExprNode, parse_expression};
use crate::parser::helper::pretty_clean_string;
use crate::parser::statement_keywords::{IfStatement, parse_if_statement, parse_return};
use crate::parser::tag_types::enum_parser::{EnumMember, parse_enum_keyword};
use crate::parser::tag_types::helper::{TagType, is_tag_type_keyword, parse_vars_after_type};
use crate::parser::tag_types::struct_parser::{StructMember, parse_struct_keyword};
use crate::parser::tag_types::union_parser::{UnionMember, parse_union_keyword};
use crate::parser::type_parser::{TypeNode, parse_type};
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

#[derive(Clone)]
pub enum GlobalNode {
    Function {
        signature: Box<TypeNode>,
        body: Option<StatementNode>,
    },

    Initalizer {
        var_type: TypeNode,
        r_value: Option<ExprNode>,
    },

    Union(TagType<UnionMember>),
    Struct(TagType<StructMember>),
    Enum(TagType<EnumMember>),
    // Typedef is on an eternal todo list
    // It'll be done, just not right now...
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

        let str_indent = " ".repeat(indentation);

        match self {
            Self::Function { signature, body } => {
                output.push_str(&signature.to_string());

                if let Some(body) = body {
                    output.pop();
                    output.push_str(&format!("\n{}", body.display(indentation + 2)));
                    output.push_str(")");
                }
            }

            Self::Initalizer { var_type, r_value } => {
                output.push_str(&format!("{str_indent}(Variable {var_type}"));

                if let Some(expression) = r_value.clone() {
                    output.push_str(&format!("{}", &expression.display(indentation + 2)));
                }

                output.push_str(")");
            }

            Self::Struct(data) => {
                output.push_str(&data.display(indentation));
            }

            Self::Enum(data) => {
                output.push_str(&data.display(indentation));
            }

            Self::Union(data) => {
                output.push_str(&data.display(indentation));
            }
        }

        output
    }
}

#[derive(Clone)]
pub enum StatementNode {
    // block, expression, if, switch, while, do, for, return, break, continue, goto, label, case, default
    Block { statements: Vec<StatementNode> },

    General(Box<GlobalNode>),
    Expression(ExprNode),

    Return(Option<ExprNode>),
    If(Box<IfStatement>),

    Semicolon,
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
        let final_str = Self::display(self, 0);

        write!(display, "{final_str}")
    }
}

impl StatementNode {
    pub fn display(&self, indentation: usize) -> String {
        let mut output = String::new();
        let str_indent = " ".repeat(indentation);

        match self {
            Self::Block { statements } => {
                for (i, statement) in statements.iter().enumerate() {
                    output.push_str(&format!("{}", statement.display(indentation)));
                    if i != statements.len() - 1 {
                        output.push('\n');
                    }
                }
            }

            Self::Expression(expr) => {
                output.push_str(&format!(
                    "{str_indent}(Expr\n{})",
                    expr.clone().display(indentation + 2)
                ));
            }

            Self::General(global_node) => {
                output.push_str(&global_node.display(indentation));
            }

            Self::Return(expr) => {
                output.push_str(&format!("{str_indent}(Return"));

                if let Some(expr) = expr {
                    output.push_str(&format!(" {}", &pretty_clean_string(&expr.to_string())));
                }

                output.push(')');
            }

            Self::If(if_statement) => {
                output.push_str(&if_statement.display(indentation));
            }

            Self::Semicolon => {
                output.push_str(&format!("{str_indent}(Op ;)"));
            }
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
            TokenTypes::Keyword(keyword) => match keyword {
                KeywordTypes::Struct => root.0.extend(parse_struct_keyword(lexer)?),
                KeywordTypes::Enum => root.0.extend(parse_enum_keyword(lexer)?),
                KeywordTypes::Union => root.0.extend(parse_union_keyword(lexer)?),
                _ => todo!(),
            },

            TokenTypes::DataType(_) => {
                root.0.extend(parse_data_type(lexer)?);
            }

            TokenTypes::Semicolon => {
                lexer.advance();
            }

            _ => unimplemented!(),
        }
    }

    Ok(root)
}

fn parse_data_type(lexer: &mut Lexer) -> Result<Vec<GlobalNode>, String> {
    if is_tag_type_keyword(lexer, &KeywordTypes::Struct)? {
        return parse_struct_keyword(lexer);
    }

    if is_tag_type_keyword(lexer, &KeywordTypes::Enum)? {
        return parse_enum_keyword(lexer);
    }

    if is_tag_type_keyword(lexer, &KeywordTypes::Union)? {
        return parse_union_keyword(lexer);
    }

    Ok(parse_function_or_var(lexer)?)
}

fn parse_function_or_var(lexer: &mut Lexer) -> Result<Vec<GlobalNode>, String> {
    lexer.set_flag();

    let type_parsed = parse_type(lexer)?;
    let is_function = matches!(type_parsed, TypeNode::Function { .. });

    if is_function {
        return Ok(vec![parse_function(lexer, &type_parsed)?]);
    }

    lexer.recede_to_flag();
    let variables = parse_variable_statement(lexer)?;

    Ok(variables)
}

fn parse_function(lexer: &mut Lexer, signature: &TypeNode) -> Result<GlobalNode, String> {
    let next_token = lexer.force_peek(
        "Expected semicolon or left curly brace after function signature, got nothing",
    )?;

    if matches!(next_token, TokenTypes::Semicolon) {
        lexer.advance();
        return Ok(GlobalNode::Function {
            signature: Box::new(signature.clone()),
            body: None,
        });
    }

    lexer.expect(|x| matches!(x, TokenTypes::LCurlyBrace))?;

    let body = parse_block(lexer)?;

    Ok(GlobalNode::Function {
        signature: Box::new(signature.clone()),
        body: Some(body),
    })
}

pub fn is_expression(token: &TokenTypes) -> bool {
    match token {
        TokenTypes::Literal(_) => true,
        TokenTypes::Operator(OperatorTypes::LParen) => true,
        TokenTypes::Identifier(_) => true,
        TokenTypes::Operator(op) if op.potential_unary() => true,
        TokenTypes::Keyword(KeywordTypes::Sizeof) => true,
        _ => false,
    }
}

/// Parses the statements within a block
/// This includes anything between a left and right curly brace that is not attached to a tag type
pub fn parse_block(lexer: &mut Lexer) -> Result<StatementNode, String> {
    let mut block = Vec::new();

    let to_statement = |x: Vec<GlobalNode>| -> Vec<StatementNode> {
        x.iter()
            .map(|x| StatementNode::General(Box::new(x.clone())))
            .collect()
    };

    while let Some(token) = lexer.peek()
        && !matches!(token, TokenTypes::RCurlyBrace)
    {
        match token {
            TokenTypes::DataType(_) => {
                block.extend(to_statement(parse_data_type(lexer)?));
            }

            x if is_expression(&x) => {
                block.push(StatementNode::Expression(parse_expression(lexer, 0)?));
                lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;
            }

            TokenTypes::Keyword(keyword) => match keyword {
                KeywordTypes::Struct => block.extend(to_statement(parse_struct_keyword(lexer)?)),
                KeywordTypes::Enum => block.extend(to_statement(parse_enum_keyword(lexer)?)),
                KeywordTypes::Union => block.extend(to_statement(parse_union_keyword(lexer)?)),
                KeywordTypes::Return => block.push(parse_return(lexer)?),
                KeywordTypes::If => block.push(parse_if_statement(lexer)?),
                _ => todo!(),
            },

            TokenTypes::Semicolon => {
                lexer.advance();
                block.push(StatementNode::Semicolon);
            }

            _ => todo!(),
        }
    }

    lexer.expect(|x| matches!(x, TokenTypes::RCurlyBrace))?;

    Ok(StatementNode::Block { statements: block })
}

/// A high level variable parser
/// Does not support struct parsing
pub fn parse_variable_statement(lexer: &mut Lexer) -> Result<Vec<GlobalNode>, String> {
    let mut var_type = parse_type(lexer)?;

    let next_token = lexer.force_peek("Expected end of var, got nothing")?;

    if matches!(next_token, TokenTypes::Semicolon) {
        lexer.advance();

        let final_var = GlobalNode::Initalizer {
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

        let first_var = GlobalNode::Initalizer {
            var_type: var_type.clone(),
            r_value: Some(parse_expression(lexer, 3)?),
        };

        all_vars.push(first_var);
    } else if matches!(next_token, TokenTypes::Operator(OperatorTypes::Comma)) {
        let first_var = GlobalNode::Initalizer {
            var_type: var_type.clone(),
            r_value: None,
        };

        all_vars.push(first_var);
    }

    let additional_vars = parse_vars_after_type::<false>(lexer, &var_type.get_most_nested_layer())?;

    all_vars.extend(additional_vars);

    Ok(all_vars)
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
