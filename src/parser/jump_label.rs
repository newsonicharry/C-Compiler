use crate::lexer::language_features::KeywordTypes;
use crate::lexer::language_features::OperatorTypes;
use crate::lexer::lexer::TokenTypes;
use crate::parser::expression_parser::ExprNode;
use crate::parser::helper::pretty_clean_string;
use crate::parser::nodes::StatementNode;
use crate::parser::parser::Parser;
use std::fmt::Display;

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

impl Parser {
    pub fn parse_goto_jump_label(&mut self) -> Result<StatementNode, String> {
        let label = self.lexer.expect_extract(|x| match x {
            TokenTypes::Identifier(label) => Some(label),
            _ => None,
        })?;

        self.lexer
            .expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::Colon)))?;

        Ok(StatementNode::JumpLabel(JumpLabel::Goto(label)))
    }

    pub fn parse_default_jump_label(&mut self) -> Result<StatementNode, String> {
        self.lexer
            .expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::Default)))?;
        self.lexer
            .expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::Colon)))?;

        Ok(StatementNode::JumpLabel(JumpLabel::DefaultCase))
    }

    pub fn parse_case_jump_label(&mut self) -> Result<StatementNode, String> {
        self.lexer
            .expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::Case)))?;

        let case_value = self.parse_expression(4)?;

        self.lexer
            .expect(|x| matches!(x, TokenTypes::Operator(OperatorTypes::Colon)))?;

        Ok(StatementNode::JumpLabel(JumpLabel::Case(case_value)))
    }

    pub fn parse_goto(&mut self) -> Result<StatementNode, String> {
        self.lexer
            .expect(|x| matches!(x, TokenTypes::Keyword(KeywordTypes::Goto)))?;

        let label = self.lexer.expect_extract(|x| match x {
            TokenTypes::Identifier(label) => Some(label),
            _ => None,
        })?;

        self.lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

        Ok(StatementNode::GotoStatement(label))
    }
}
