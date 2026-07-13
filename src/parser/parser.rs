use crate::lexer::language_features::KeywordTypes;
use crate::lexer::language_features::{AssignmentTypes, OperatorTypes};
use crate::lexer::lexer::{Lexer, TokenTypes};
use crate::parser::helper::to_statement;
use crate::parser::nodes::{GlobalNode, Root, StatementNode};
use crate::parser::tag_types::helper::TagTypeKind;
use crate::parser::type_parser::TypeNode;
use crate::semantics::semantics::{SemanticInfo, Semantics};

pub struct Parser {
    pub lexer: Lexer,
    pub semantics: Semantics,
}

impl Parser {
    pub fn new(lexer: &Lexer) -> Parser {
        Parser {
            lexer: lexer.clone(),
            semantics: Semantics::default(),
        }
    }

    pub fn parse_program(&mut self) -> Result<Root, String> {
        let mut root = Root(Vec::new());

        while let Some(token) = self.lexer.peek() {
            match token {
                TokenTypes::Keyword(keyword) => match keyword {
                    KeywordTypes::Struct => root.0.extend(self.parse_struct_keyword()?),
                    KeywordTypes::Enum => root.0.extend(self.parse_enum_keyword()?),
                    KeywordTypes::Union => root.0.extend(self.parse_union_keyword()?),
                    _ => todo!(),
                },

                TokenTypes::Identifier(identifier) => {
                    if self.semantics.check_typedef(&identifier).is_some() {
                        root.0.extend(self.parse_data_type()?);
                    }
                }

                TokenTypes::DataType(_) => {
                    root.0.extend(self.parse_data_type()?);
                }

                TokenTypes::Semicolon => {
                    self.lexer.advance();
                }

                _ => unimplemented!(),
            }
        }

        Ok(root)
    }

    fn parse_data_type(&mut self) -> Result<Vec<GlobalNode>, String> {
        match self.is_tag_type_keyword()? {
            Some(value) => match value {
                TagTypeKind::Struct => self.parse_struct_keyword(),
                TagTypeKind::Union => self.parse_union_keyword(),
                TagTypeKind::Enum => self.parse_enum_keyword(),
            },

            None => self.parse_function_or_var(),
        }
    }

    fn parse_function_or_var(&mut self) -> Result<Vec<GlobalNode>, String> {
        self.lexer.set_flag();
        let type_parsed = self.parse_type()?;
        let is_function = matches!(type_parsed, TypeNode::Function { .. });

        if is_function {
            return Ok(self.parse_function(&type_parsed)?);
        }

        self.lexer.recede_to_flag();
        let variables = self.parse_variable_statement()?;

        Ok(variables)
    }

    fn parse_function(&mut self, signature: &TypeNode) -> Result<Vec<GlobalNode>, String> {
        let next_token = self.lexer.force_peek(
            "Expected semicolon or left curly brace after function signature, got nothing",
        )?;

        if matches!(next_token, TokenTypes::Semicolon) {
            self.lexer.advance();
            let function = GlobalNode::Function {
                signature: Box::new(signature.clone()),
                body: None,
                semantic_info: SemanticInfo::default(),
            };

            // functions can be typedefed
            if self.is_typedef_analysis(&function)? {
                return Ok(vec![]);
            }

            return Ok(vec![function]);
        }

        self.lexer
            .expect(|x| matches!(x, TokenTypes::LCurlyBrace))?;
        let body = self.parse_block()?;

        let function = GlobalNode::Function {
            signature: Box::new(signature.clone()),
            body: Some(body),
            semantic_info: SemanticInfo::default(),
        };

        // let an error propagate if the final function is a typedef
        // if theres a body then a typedef is illegal
        self.is_typedef_analysis(&function)?;

        Ok(vec![function])
    }

    /// Parses the statements within a block
    /// This includes anything between a left and right curly brace that is not attached to a tag type
    pub fn parse_block(&mut self) -> Result<StatementNode, String> {
        self.semantics.enter_scope();

        let mut block = Vec::new();
        while let Some(token) = self.lexer.peek()
            && !matches!(token, TokenTypes::RCurlyBrace)
        {
            match token {
                TokenTypes::Keyword(KeywordTypes::Struct) => {
                    block.extend(to_statement(self.parse_struct_keyword()?))
                }
                TokenTypes::Keyword(KeywordTypes::Enum) => {
                    block.extend(to_statement(self.parse_enum_keyword()?))
                }

                TokenTypes::Keyword(KeywordTypes::Union) => {
                    block.extend(to_statement(self.parse_union_keyword()?))
                }

                TokenTypes::DataType(_) => {
                    block.extend(to_statement(self.parse_variable_statement()?))
                }

                TokenTypes::LCurlyBrace => {
                    self.lexer.advance();
                    block.push(self.parse_block()?);
                }

                _ => block.push(self.parse_single_statement()?),
            }
        }

        self.lexer
            .expect(|x| matches!(x, TokenTypes::RCurlyBrace))?;

        self.semantics.leave_scope();

        Ok(StatementNode::Block { statements: block })
    }

    /// A high level variable parser
    /// Does not support struct parsing
    pub fn parse_variable_statement(&mut self) -> Result<Vec<GlobalNode>, String> {
        let mut var_type = self.parse_type()?;
        let next_token = self.lexer.force_peek("Expected end of var, got nothing")?;
        let mut all_vars = vec![];

        if matches!(next_token, TokenTypes::Semicolon) {
            self.lexer.advance();
            let final_var = GlobalNode::Initalizer {
                var_type: var_type.clone(),
                r_value: None,
                semantic_info: SemanticInfo::default(),
            };
            if self.is_typedef_analysis(&final_var)? {
                return Ok(vec![]);
            }

            return Ok(vec![final_var]);
        } else if matches!(
            next_token,
            TokenTypes::Assignment(AssignmentTypes::SimpleAssignment)
        ) {
            self.lexer.advance();

            let first_var = GlobalNode::Initalizer {
                var_type: var_type.clone(),
                r_value: Some(self.parse_expression(3)?),
                semantic_info: SemanticInfo::default(),
            };

            all_vars.push(first_var);
        } else if matches!(next_token, TokenTypes::Operator(OperatorTypes::Comma)) {
            let first_var = GlobalNode::Initalizer {
                var_type: var_type.clone(),
                r_value: None,
                semantic_info: SemanticInfo::default(),
            };

            all_vars.push(first_var);
        } else {
            return Err(format!(
                "Expected variable declaration to have an ending semicolon, got token of type {next_token}",
            ));
        }

        let additional_vars =
            self.parse_vars_after_type::<false>(&var_type.get_most_nested_layer())?;

        all_vars.extend(additional_vars);

        // if one variable is a typedef all must be since they all share the same type
        let mut are_typedefs = false;
        for var in &all_vars {
            // despite that is_typedef_analysis still needs to run to add it to the symbol table
            if self.is_typedef_analysis(&var)? {
                are_typedefs = true;
                continue;
            }

            // if not a typedef add the variable to the symbol table
            let GlobalNode::Initalizer { var_type, .. } = var else {
                unreachable!();
            };
        }

        // we don't add typedefs to the ast
        if are_typedefs {
            return Ok(vec![]);
        }

        Ok(all_vars)
    }
}
