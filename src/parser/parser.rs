use crate::lexer::language_features::{AssignmentTypes, KeywordTypes, OperatorTypes};
use crate::lexer::lexer::{Lexer, TokenTypes};
use crate::parser::expression_parser::{ExprNode, parse_expression};
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

    Struct {
        name: Option<String>, // can be anonymous
        members: Vec<TypeNode>,
    },

    Union {
        name: Option<String>,
        members: Vec<TypeNode>,
    },

    Enum {
        name: Option<String>,
        members: Vec<EnumMember>,
    },

    Typedef {}, // Todo, don't want to even touch these yet
}

impl Display for GlobalNode {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let final_str = Self::display(self);

        write!(display, "{final_str}")
    }
}

impl GlobalNode {
    fn display(&self) -> String {
        let mut output = String::new();

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

            _ => todo!(),
        }

        output
    }
}

pub enum StatementNode {
    // block, expression, if, switch, while, do, for, return, break, continue, goto, label, case, default
    Block {
        statements: Vec<StatementNode>,
    },

    Expression {
        var_type: TypeNode,
        r_value: Option<(AssignmentTypes, ExprNode)>,
    },
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

                if let Some((assign_op, expression)) = r_value {
                    output.push_str(&format!(
                        " {assign_op} {}",
                        expression
                            .to_string()
                            .chars()
                            .filter(|x| *x != '\n')
                            .collect::<String>(),
                    ));
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
            TokenTypes::DataType(_) => {
                root.0.push(parse_function_or_var(lexer)?);
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
        false => parse_var(lexer),
    }
}

fn parse_var(lexer: &mut Lexer) -> Result<GlobalNode, String> {
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

        return Ok(GlobalNode::Variable {
            expr_statement: final_var,
        });
    }

    let assign_op = lexer.expect_extract(|x| match x {
        TokenTypes::Assignment(assign_op) => Some(assign_op),
        _ => None,
    })?;

    let expression = parse_expression(lexer, 0);

    lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

    let final_var = StatementNode::Expression {
        var_type,
        r_value: Some((assign_op, expression)),
    };

    return Ok(GlobalNode::Variable {
        expr_statement: final_var,
    });
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
