use crate::lexer::language_features::KeywordTypes;
use crate::lexer::language_features::OperatorTypes;
use crate::lexer::lexer::{Lexer, TokenTypes};
use crate::parser::expression_parser::{ExprNode, parse_expression};
use crate::parser::helper::pretty_clean_string;
use crate::parser::helper::to_statement;
use crate::parser::parser::StatementNode;
use crate::parser::parser::is_expression;
use crate::parser::parser::parse_block;
use crate::parser::parser::parse_variable_statement;
use std::fmt::Display;

pub fn parse_return(lexer: &mut Lexer) -> Result<StatementNode, String> {
    lexer.expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::Return)))?;

    let next_token = lexer.force_peek("Unexpected end to return keyword")?;

    let final_return = match next_token {
        TokenTypes::Semicolon => StatementNode::Return(None),

        _ => {
            let expr = parse_expression(lexer, 0)?;
            StatementNode::Return(Some(expr))
        }
    };

    lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

    Ok(final_return)
}

pub fn parse_break(lexer: &mut Lexer) -> Result<StatementNode, String> {
    lexer.expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::Break)))?;
    lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;
    Ok(StatementNode::Break)
}

pub fn parse_continue(lexer: &mut Lexer) -> Result<StatementNode, String> {
    lexer.expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::Continue)))?;
    lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;
    Ok(StatementNode::Continue)
}

fn parse_single_or_multi_statement_body(lexer: &mut Lexer) -> Result<Box<StatementNode>, String> {
    let next_token = lexer.force_peek("Expected while loop body, got nothing")?;

    let body;
    if matches!(next_token, TokenTypes::LCurlyBrace) {
        lexer.advance();
        body = Box::new(parse_block(lexer)?);
    } else {
        body = Box::new(parse_single_statement(lexer)?);
    }

    Ok(body)
}

pub fn parse_while(lexer: &mut Lexer) -> Result<StatementNode, String> {
    lexer.expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::While)))?;
    lexer.expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::LParen)))?;

    let conditional = parse_expression(lexer, 0)?;

    lexer.expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::RParen)))?;

    let body = parse_single_or_multi_statement_body(lexer)?;

    Ok(StatementNode::While { conditional, body })
}

pub fn parse_do_while(lexer: &mut Lexer) -> Result<StatementNode, String> {
    lexer.expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::Do)))?;

    let body = parse_single_or_multi_statement_body(lexer)?;

    lexer.expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::While)))?;
    lexer.expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::LParen)))?;

    let conditional = parse_expression(lexer, 0)?;

    lexer.expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::RParen)))?;
    lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

    Ok(StatementNode::DoWhile { conditional, body })
}

pub fn parse_for(lexer: &mut Lexer) -> Result<StatementNode, String> {
    lexer.expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::For)))?;
    lexer.expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::LParen)))?;

    let mut parse_section = |parse_var: bool| -> Result<Option<StatementNode>, String> {
        let next_token = lexer.force_peek("Expected next token in for loop, got nothing")?;
        let clause = match next_token {
            TokenTypes::DataType(_) => {
                if !parse_var {
                    return Err(String::from("Unexpected data type in for loop clause"));
                }

                Some(StatementNode::Block {
                    statements: to_statement(parse_variable_statement(lexer)?),
                })
            }

            TokenTypes::Semicolon => {
                lexer.advance();
                None
            }

            _ => {
                let clause = StatementNode::Expression(parse_expression(lexer, 0)?);
                if !matches!(
                    lexer.force_peek("Expected next token in for loop")?,
                    TokenTypes::Operator(OperatorTypes::RParen)
                ) {
                    lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;
                }
                Some(clause)
            }
        };

        Ok(clause)
    };

    let init_clause = parse_section(true)?;
    let condition = parse_section(false)?;
    let iteration = parse_section(false)?;

    let convert_to_expr = |x: Option<StatementNode>| {
        if let Some(StatementNode::Expression(expr)) = x {
            return Some(expr);
        }

        None
    };
    lexer.expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::RParen)))?;

    let next_token = lexer.force_peek("Expected next token in for loop, got nothing")?;

    let body;
    if matches!(next_token, TokenTypes::LCurlyBrace) {
        lexer.advance();
        body = Box::new(parse_block(lexer)?);
    } else {
        body = Box::new(parse_single_statement(lexer)?);
    }

    Ok(StatementNode::For {
        init: init_clause.map(|x| Box::new(x)),
        condition: convert_to_expr(condition),
        iteration: convert_to_expr(iteration),
        body,
    })
}

pub fn parse_switch_case(lexer: &mut Lexer) -> Result<StatementNode, String> {
    lexer.expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::Switch)))?;
    lexer.expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::LParen)))?;

    let case_label = parse_expression(lexer, 0)?;

    lexer.expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::RParen)))?;

    let body = parse_single_or_multi_statement_body(lexer)?;

    Ok(StatementNode::Switch { case_label, body })
}

#[derive(Clone)]
pub enum JumpLabel {
    Goto(String),
    DefaultCase,
    Case(ExprNode),
}

impl Display for JumpLabel {
    fn fmt(&self, display: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let output = match self {
            JumpLabel::Goto(label) => String::from(&format!("(GotoLabel {label})")),
            JumpLabel::DefaultCase => String::from("(DefaultLabel)"),
            JumpLabel::Case(case_value) => String::from(&format!(
                "(CaseLabel {})",
                pretty_clean_string(&case_value.to_string())
            )),
        };

        write!(display, "{output}")
    }
}

fn parse_goto_jump_label(lexer: &mut Lexer) -> Result<StatementNode, String> {
    let label = lexer.expect_extract(|x| match x {
        TokenTypes::Identifier(label) => Some(label),
        _ => None,
    })?;

    lexer.expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::Colon)))?;

    Ok(StatementNode::JumpLabel(JumpLabel::Goto(label)))
}

fn parse_default_jump_label(lexer: &mut Lexer) -> Result<StatementNode, String> {
    lexer.expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::Default)))?;
    lexer.expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::Colon)))?;

    Ok(StatementNode::JumpLabel(JumpLabel::DefaultCase))
}

fn parse_case_jump_label(lexer: &mut Lexer) -> Result<StatementNode, String> {
    lexer.expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::Case)))?;

    let case_value = parse_expression(lexer, 4)?;

    lexer.expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::Colon)))?;

    Ok(StatementNode::JumpLabel(JumpLabel::Case(case_value)))
}

fn parse_goto(lexer: &mut Lexer) -> Result<StatementNode, String> {
    lexer.expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::Goto)))?;

    let label = lexer.expect_extract(|x| match x {
        TokenTypes::Identifier(label) => Some(label),
        _ => None,
    })?;

    lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

    Ok(StatementNode::GotoStatement(label))
}

#[derive(Clone)]
pub enum IfStatement {
    If {
        conditional: ExprNode,
        body: Box<StatementNode>, // either an expression or block
        chain: Option<Box<IfStatement>>,
    },
    ElseIf {
        conditional: ExprNode,
        body: Box<StatementNode>,
        chain: Option<Box<IfStatement>>,
    },
    Else(StatementNode),
}

impl IfStatement {
    pub fn display(&self, indentation: usize) -> String {
        let mut output = String::new();
        let indent_str = " ".repeat(indentation);
        let next_indent_str = " ".repeat(indentation + 2);

        match self {
            Self::If { .. } => output.push_str(&format!("{indent_str}(If")),
            Self::ElseIf { .. } => output.push_str(&format!("{indent_str}(ElseIf")),
            Self::Else { .. } => output.push_str(&format!("{indent_str}(Else")),
        }

        match self {
            Self::If {
                conditional,
                body,
                chain,
            }
            | Self::ElseIf {
                conditional,
                body,
                chain,
            } => {
                output.push_str(&format!(
                    "\n{next_indent_str}(Condition \n{})\n{next_indent_str}(Body\n{}\n{next_indent_str})",
                    &conditional.clone().display(indentation + 4),
                    &body.clone().display(indentation + 4)
                ));

                if let Some(chain) = chain {
                    output.push_str(&format!(
                        "\n{next_indent_str}(Chain\n{}\n{next_indent_str})",
                        chain.display(indentation + 4)
                    ));
                }

                output.push(')');
            }
            Self::Else(body) => {
                output.push_str(&format!(
                    "\n{next_indent_str}(Body\n{}\n{next_indent_str})",
                    &body.clone().display(indentation + 4)
                ));
            }
        }

        output
    }
}

pub fn parse_single_statement(lexer: &mut Lexer) -> Result<StatementNode, String> {
    let next_token = lexer.force_peek("Expected if statement to have a body, got nothing")?;

    let statement = match next_token {
        TokenTypes::Identifier(_) => {
            lexer.set_flag();
            let goto_label = parse_goto_jump_label(lexer);
            if goto_label.is_ok() {
                return goto_label;
            }
            lexer.recede_to_flag();

            let statement = StatementNode::Expression(parse_expression(lexer, 0)?);
            lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

            statement
        }

        x if is_expression(&x) => {
            let statement = StatementNode::Expression(parse_expression(lexer, 0)?);
            lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

            statement
        }

        TokenTypes::Keyword(keyword) => match keyword {
            KeywordTypes::Return => parse_return(lexer)?,
            KeywordTypes::Break => parse_break(lexer)?,
            KeywordTypes::Continue => parse_continue(lexer)?,
            KeywordTypes::If => parse_if_statement(lexer)?,
            KeywordTypes::While => parse_while(lexer)?,
            KeywordTypes::Do => parse_do_while(lexer)?,
            KeywordTypes::For => parse_for(lexer)?,
            KeywordTypes::Case => parse_case_jump_label(lexer)?,
            KeywordTypes::Default => parse_default_jump_label(lexer)?,
            KeywordTypes::Switch => parse_switch_case(lexer)?,
            KeywordTypes::Goto => parse_goto(lexer)?,

            _ => return Err(format!("Expected statement got non statement of {keyword}",)),
        },

        TokenTypes::Semicolon => {
            lexer.advance();
            StatementNode::Semicolon
        }

        _ => {
            return Err(format!(
                "Unexpected token of type {next_token} for statement"
            ));
        }
    };

    Ok(statement)
}

pub fn parse_if_statement(lexer: &mut Lexer) -> Result<StatementNode, String> {
    lexer.expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::If)))?;

    lexer.expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::LParen)))?;
    let conditional = parse_expression(lexer, 0)?;
    lexer.expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::RParen)))?;

    let next_token = lexer.force_peek("Expected if statement to have a body, got nothing")?;

    let body;
    if matches!(next_token, TokenTypes::LCurlyBrace) {
        lexer.advance();
        body = Box::new(parse_block(lexer)?);
    } else {
        body = Box::new(parse_single_statement(lexer)?)
    }

    let next_token = lexer.force_peek("Unexpected end to body")?;

    let mut chain = None;
    if matches!(next_token, TokenTypes::Keyword(KeywordTypes::Else)) {
        chain = Some(Box::new(parse_else_if_statement(lexer)?));
    }

    let final_if = IfStatement::If {
        conditional,
        body,
        chain,
    };

    Ok(StatementNode::If(Box::new(final_if)))
}

fn parse_else_if_statement(lexer: &mut Lexer) -> Result<IfStatement, String> {
    lexer.expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::Else)))?;

    let next_token = lexer.force_peek("Expected else statement to have a body, got nothing")?;

    // else statement
    if !matches!(next_token, TokenTypes::Keyword(KeywordTypes::If)) {
        let next_token = lexer.force_peek("Expected else statement to have a body, got nothing")?;

        if matches!(next_token, TokenTypes::LCurlyBrace) {
            lexer.advance();
            return Ok(IfStatement::Else(parse_block(lexer)?));
        }

        return Ok(IfStatement::Else(parse_single_statement(lexer)?));
    }

    // else if
    let StatementNode::If(chained) = parse_if_statement(lexer)? else {
        unreachable!();
    };

    let IfStatement::If {
        conditional,
        body,
        chain,
    } = *chained
    else {
        unreachable!();
    };

    Ok(IfStatement::ElseIf {
        conditional,
        body,
        chain,
    })
}

#[cfg(test)]
mod tests {
    use crate::parser::helper::run_tests;
    use crate::parser::parser::parse_program;

    #[test]
    fn if_statement() {
        let test_cases = vec![
            (
                "int main(){ if (1) ; } ",
                "
                (Function main (Return (Type int))
                    (If (Condition (Num 1))
                        (Body (Op ;))))
                ",
            ),
            (
                "int main(){ if (x > 5) y++; } ",
                "
                (Function main (Return (Type int))
                    (If
                        (Condition  (Binary (Var x) (Op >) (Num 5)))
                        (Body (Expr (Postfix (Var y) (PostInc)))
                )))
                ",
            ),
            (
                "
                int main(){
                    if (x) y++;
                    else y--;
                }
                ",
                "
                (Function main (Return (Type int))
                (If
                    (Condition (Var x))
                    (Body
                        (Expr (Postfix (Var y) (PostInc))))
                    (Chain (Else
                        (Body (Expr (Postfix (Var y) (PostDec))))))
                )
                ",
            ),
            (
                "
                int main(){
                    if (a)
                        if (b)
                            c++;
                }
                ",
                "
                (Function main (Return (Type int))
                (If
                    (Condition (Var a))
                    (Body
                        (If
                            (Condition (Var b))
                            (Body (Expr (Postfix (Var c) (PostInc))))
                        )
                    )
                ))
                ",
            ),
            (
                "
                int main(){
                    if (a)
                        if (b)
                            c++;
                        else
                            d++;
                }
                ",
                "
                (Function main (Return (Type int))
                (If
                    (Condition (Var a))
                    (Body
                        (If
                            (Condition (Var b))
                            (Body (Expr (Postfix (Var c) (PostInc))))
                            (Chain
                                (Else (Body (Expr (Postfix (Var d) (PostInc))))
                    ))
                )))
                ",
            ),
            (
                "
                int main(){
                    if (a) {
                        if (b)
                            c++;
                    }
                    else {
                        d++;
                    }
                }
                ",
                "
                (Function main (Return (Type int))
                (If
                    (Condition (Var a))
                    (Body
                        (If (Condition (Var b))
                        (Body (Expr (Postfix (Var c) (PostInc))))))
                        (Chain (Else
                            (Body (Expr (Postfix (Var d) (PostInc))))))
                )
                ",
            ),
            (
                "
                int main(){
                    if (x) ;
                    else ;
                }
                ",
                "
                (Function main (Return (Type int))
                (If
                    (Condition (Var x))
                    (Body (Op ;))
                    (Chain (Else
                        (Body (Op ;))))
                )
                ",
            ),
            (
                "
                int main(){
                    if (x == 0)
                        a();
                    else if (x == 1)
                        b();
                    else if (x == 2)
                        c();
                    else
                        d();
                }
                ",
                "
                (Function main (Return (Type int))
                  (If
                    (Condition (Binary  (Var x)  (Op ==)  (Num 0)))
                    (Body (Expr (Postfix (Var a) (FuncCall))))
                    (Chain
                        (ElseIf
                            (Condition (Binary (Var x) (Op ==) (Num 1)))
                            (Body (Expr (Postfix (Var b) (FuncCall))))
                            (Chain
                                (ElseIf
                                    (Condition (Binary (Var x) (Op ==) (Num 2)))
                                    (Body (Expr (Postfix (Var c) (FuncCall))))
                                    (Chain
                                        (Else
                                            (Body (Expr (Postfix (Var d) (FuncCall)))))))))))
                ",
            ),
        ];

        run_tests(parse_program, test_cases);
    }
}
