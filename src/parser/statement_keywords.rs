use crate::lexer::language_features::KeywordTypes;
use crate::lexer::language_features::OperatorTypes;
use crate::lexer::lexer::TokenTypes;
use crate::parser::helper::is_expression;
use crate::parser::helper::to_statement;
use crate::parser::nodes::StatementNode;
use crate::parser::parser::Parser;

impl Parser {
    pub fn parse_return(&mut self) -> Result<StatementNode, String> {
        self.lexer
            .expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::Return)))?;

        let next_token = self.lexer.force_peek("Unexpected end to return keyword")?;

        let final_return = match next_token {
            TokenTypes::Semicolon => StatementNode::Return(None),

            _ => {
                let expr = self.parse_expression(0)?;
                StatementNode::Return(Some(expr))
            }
        };

        self.lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

        Ok(final_return)
    }

    pub fn parse_break(&mut self) -> Result<StatementNode, String> {
        self.lexer
            .expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::Break)))?;
        self.lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;
        Ok(StatementNode::Break)
    }

    pub fn parse_continue(&mut self) -> Result<StatementNode, String> {
        self.lexer
            .expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::Continue)))?;
        self.lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;
        Ok(StatementNode::Continue)
    }

    fn parse_single_or_multi_statement_body(&mut self) -> Result<Box<StatementNode>, String> {
        let next_token = self
            .lexer
            .force_peek("Expected while loop body, got nothing")?;

        let body;
        if matches!(next_token, TokenTypes::LCurlyBrace) {
            self.lexer.advance();
            body = Box::new(self.parse_block()?);
        } else {
            body = Box::new(self.parse_single_statement()?);
        }

        Ok(body)
    }

    pub fn parse_while(&mut self) -> Result<StatementNode, String> {
        self.lexer
            .expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::While)))?;
        self.lexer
            .expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::LParen)))?;

        let conditional = self.parse_expression(0)?;

        self.lexer
            .expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::RParen)))?;

        let body = self.parse_single_or_multi_statement_body()?;

        Ok(StatementNode::While { conditional, body })
    }

    pub fn parse_do_while(&mut self) -> Result<StatementNode, String> {
        self.lexer
            .expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::Do)))?;

        let body = self.parse_single_or_multi_statement_body()?;

        self.lexer
            .expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::While)))?;
        self.lexer
            .expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::LParen)))?;

        let conditional = self.parse_expression(0)?;

        self.lexer
            .expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::RParen)))?;
        self.lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

        Ok(StatementNode::DoWhile { conditional, body })
    }

    pub fn parse_for(&mut self) -> Result<StatementNode, String> {
        self.lexer
            .expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::For)))?;
        self.lexer
            .expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::LParen)))?;

        // the init clause means there will always be a scope, as one can define a variable there
        // though it is possible that this scope is empty, such as a for(;;) or an ANSI for loop
        self.semantics.enter_scope();

        let mut parse_section = |parse_var: bool| -> Result<Option<StatementNode>, String> {
            let next_token = self
                .lexer
                .force_peek("Expected next token in for loop, got nothing")?;
            let clause = match next_token {
                TokenTypes::DataType(_) => {
                    if !parse_var {
                        return Err(String::from("Unexpected data type in for loop clause"));
                    }

                    Some(StatementNode::Block {
                        statements: to_statement(self.parse_variable_statement()?),
                        scope_id: self.semantics.curr_scope_id(),
                    })
                }

                TokenTypes::Semicolon => {
                    self.lexer.advance();
                    None
                }

                _ => {
                    let clause = StatementNode::Expression(self.parse_expression(0)?);
                    if !matches!(
                        self.lexer.force_peek("Expected next token in for loop")?,
                        TokenTypes::Operator(OperatorTypes::RParen)
                    ) {
                        self.lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;
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
        self.lexer
            .expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::RParen)))?;

        let next_token = self
            .lexer
            .force_peek("Expected next token in for loop, got nothing")?;

        let body;
        if matches!(next_token, TokenTypes::LCurlyBrace) {
            self.lexer.advance();
            body = Box::new(self.parse_block()?);
        } else {
            body = Box::new(self.parse_single_statement()?);
        }

        self.semantics.leave_scope();

        Ok(StatementNode::For {
            init: init_clause.map(|x| Box::new(x)),
            condition: convert_to_expr(condition),
            iteration: convert_to_expr(iteration),
            body,
        })
    }

    pub fn parse_switch_case(&mut self) -> Result<StatementNode, String> {
        self.lexer
            .expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::Switch)))?;
        self.lexer
            .expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::LParen)))?;

        let case_label = self.parse_expression(0)?;

        self.lexer
            .expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::RParen)))?;

        let body = self.parse_single_or_multi_statement_body()?;

        Ok(StatementNode::Switch { case_label, body })
    }

    pub fn parse_single_statement(&mut self) -> Result<StatementNode, String> {
        let next_token = self
            .lexer
            .force_peek("Expected if statement to have a body, got nothing")?;

        let statement = match next_token {
            TokenTypes::Identifier(_) => {
                self.lexer.set_flag();
                let goto_label = self.parse_goto_jump_label();
                if goto_label.is_ok() {
                    return goto_label;
                }
                self.lexer.recede_to_flag();

                let statement = StatementNode::Expression(self.parse_expression(0)?);
                self.lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

                statement
            }

            x if is_expression(&x) => {
                let statement = StatementNode::Expression(self.parse_expression(0)?);
                self.lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

                statement
            }

            TokenTypes::Keyword(keyword) => match keyword {
                KeywordTypes::Return => self.parse_return()?,
                KeywordTypes::Break => self.parse_break()?,
                KeywordTypes::Continue => self.parse_continue()?,
                KeywordTypes::If => self.parse_if_statement()?,
                KeywordTypes::While => self.parse_while()?,
                KeywordTypes::Do => self.parse_do_while()?,
                KeywordTypes::For => self.parse_for()?,
                KeywordTypes::Case => self.parse_case_jump_label()?,
                KeywordTypes::Default => self.parse_default_jump_label()?,
                KeywordTypes::Switch => self.parse_switch_case()?,
                KeywordTypes::Goto => self.parse_goto()?,

                _ => return Err(format!("Expected statement got non statement of {keyword}",)),
            },

            TokenTypes::Semicolon => {
                self.lexer.advance();
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
}
