use crate::lexer::language_features::{AssignmentTypes, DataTypes, KeywordTypes, OperatorTypes};
use crate::lexer::lexer::TokenTypes;
use crate::parser::expression_parser::ExprNode;
use crate::parser::helper::pretty_clean_string;
use crate::parser::nodes::{GlobalNode, IndentDisplay};
use crate::parser::parser::Parser;
use crate::parser::type_parser::TypeNode;

#[derive(Clone)]
pub enum TagTypeMember {
    StructMember {
        item_type: TypeNode,
        bit_field: Option<u64>,
    },
    UnionMember {
        item_type: TypeNode,
    },
    EnumMember {
        name: String,
        value: Option<ExprNode>,
    },
    TagType(TagTypeData),
}

impl IndentDisplay for TagTypeMember {
    fn indent_display(&self, indent: usize) -> String {
        match self {
            Self::StructMember { .. } => self.struct_display_helper(indent),
            Self::UnionMember { .. } => self.union_display_helper(indent),
            Self::EnumMember { .. } => self.enum_display_helper(indent),
            Self::TagType(tag_type_data) => tag_type_data.indent_display(indent + 2),
        }
    }
}

impl TagTypeMember {
    fn struct_display_helper(&self, indent: usize) -> String {
        let Self::StructMember {
            item_type,
            bit_field,
        } = self
        else {
            unreachable!()
        };

        let mut output = String::new();
        let indent_str = " ".repeat(indent);

        output += &format!("{indent_str}(Member {}", item_type);

        if let Some(bitfield) = bit_field {
            output.push_str(&format!("(Bitfield {})", bitfield.to_string()));
        }

        output.push(')');

        output
    }

    fn union_display_helper(&self, indent: usize) -> String {
        let Self::UnionMember { item_type } = self else {
            unreachable!()
        };

        let indent_str = " ".repeat(indent);
        format!("{indent_str}(Member {})", item_type)
    }

    fn enum_display_helper(&self, indent: usize) -> String {
        let Self::EnumMember { name, value } = self else {
            unreachable!()
        };

        let indent_str = " ".repeat(indent);
        let mut output = format!("{indent_str}(Member {}", name);

        if let Some(enum_value) = &value {
            output.push_str(&format!(
                " (Value {})",
                pretty_clean_string(&enum_value.to_string())
            ));
        }

        output.push(')');
        output
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TagTypeKind {
    Struct,
    Union,
    Enum,
}

impl From<&KeywordTypes> for TagTypeKind {
    fn from(value: &KeywordTypes) -> Self {
        match value {
            KeywordTypes::Struct => Self::Struct,
            KeywordTypes::Enum => Self::Enum,
            KeywordTypes::Union => Self::Union,
            _ => panic!("Expected keyword type struct, enum or union for tag type"),
        }
    }
}

#[derive(Clone)]
pub struct TagTypeData {
    pub kind: TagTypeKind,
    pub is_defined: bool,
    pub name: Option<String>,
    pub members: Vec<TagTypeMember>,
}

impl TagTypeData {
    pub fn as_type(&self, properties: &Vec<DataTypes>) -> TypeNode {
        if self.name.is_none() {
            panic!("TagTypeData::as_type can only be called after tag type is given a name")
        }

        let name = self.name.as_ref().unwrap();

        TypeNode::TagType {
            kind: self.kind.clone(),
            name: name.to_string(),
            qualifiers: properties.clone(),
        }
    }

    fn indent_display_helper(&self, tag_type_name: &str, indent: usize) -> String {
        let mut output = String::new();
        let indent_str = " ".repeat(indent);
        output.push_str(&format!("{indent_str}({tag_type_name}"));
        if let Some(name) = self.name.clone() {
            output.push_str(&format!(" {name}"));
        }

        if self.is_defined {
            output.push_str(&format!(" (Members"));

            for member in &self.members {
                output.push_str(&format!("\n{}", member.indent_display(indent + 2)));
            }

            if !self.members.is_empty() {
                output.push_str(&format!("\n{indent_str}"));
            }
            output.push_str(")");
        }

        output.push_str(")");

        output
    }
}

impl IndentDisplay for TagTypeData {
    fn indent_display(&self, indent: usize) -> String {
        match self.kind {
            TagTypeKind::Struct => self.indent_display_helper("Struct", indent),
            TagTypeKind::Enum => self.indent_display_helper("Enum", indent),
            TagTypeKind::Union => self.indent_display_helper("Union", indent),
        }
    }
}

#[derive(Debug)]
pub enum TagKeywordUsage {
    Definition,
    Declaration,
    Variable,
}

impl Parser {
    /// Parses the qualifiers of a tag type after its been defined
    /// (e.g struct Point{int x;} const volatile p, q;) where the const volatile portion is parsed
    fn parse_tag_type_qualifiers(&mut self) -> Result<Vec<DataTypes>, String> {
        let mut qualifiers = Vec::new();
        while let Some(TokenTypes::DataType(data_type)) = self.lexer.peek() {
            if data_type.is_qualifier() || data_type.is_storage_specifier() {
                qualifiers.push(data_type);
                self.lexer.advance();
                continue;
            }

            return Err(format!(
                "Expected data type after tag type to be a qualifier, got {data_type}"
            ));
        }

        Ok(qualifiers)
    }

    /// Determines how the tag type keyword is being used
    /// This could be eihter as a defintion, declaration or variable
    pub fn tag_type_keyword_usage(&mut self) -> Result<TagKeywordUsage, String> {
        self.lexer.set_flag();

        // the qualifiers don't matter we just want to skip them here
        self.parse_tag_type_qualifiers()?;

        self.lexer.advance(); // move past the tag type keyword

        // Move past the tag type name if it exists
        if let Some(TokenTypes::Identifier(_)) = self.lexer.peek() {
            self.lexer.next();
        }

        self.parse_tag_type_qualifiers()?;

        let next_token = self
            .lexer
            .force_peek("Expected next token in tag type definition, got nothing")?;

        // make sure we don't mess up the parsing for our parsing functions
        self.lexer.recede_to_flag();

        // if its a variable
        // includes left parenthesis and start because it could be a function pointer or pointer
        if matches!(next_token, TokenTypes::Identifier(_))
            || matches!(next_token, TokenTypes::Operator(OperatorTypes::LParen))
            || matches!(next_token, TokenTypes::Operator(OperatorTypes::Star))
        {
            return Ok(TagKeywordUsage::Variable);
        }

        if matches!(next_token, TokenTypes::LCurlyBrace) {
            return Ok(TagKeywordUsage::Definition);
        }

        if matches!(next_token, TokenTypes::Semicolon) {
            return Ok(TagKeywordUsage::Declaration);
        }

        Err(String::from(&format!(
            "Unexpected next token {next_token}, expected tag type variable, definition or declaration",
        )))
    }

    /// Determines if a sequence of tokens uses a certain tag type
    /// Used within the main parser to determine if it should go to the struct parser
    pub fn is_tag_type_keyword(&mut self) -> Result<Option<TagTypeKind>, String> {
        self.lexer.set_flag();
        let _ = self.parse_tag_type_qualifiers();

        let curr_token = self.lexer.force_peek("Expected next token, got nothing")?;
        if let TokenTypes::Keyword(keyword) = curr_token {
            self.lexer.recede_to_flag();
            match keyword {
                KeywordTypes::Struct => return Ok(Some(TagTypeKind::Struct)),
                KeywordTypes::Union => return Ok(Some(TagTypeKind::Union)),
                KeywordTypes::Enum => return Ok(Some(TagTypeKind::Enum)),

                _ => {}
            };
        }

        if matches!(curr_token, TokenTypes::Keyword(_)) {
            self.lexer.recede_to_flag();
            return Ok(None);
        }

        let parsed_type = self.parse_type()?;

        self.lexer.recede_to_flag();

        if let Some(tag_type_kind) = parsed_type.contains_tag_type() {
            return Ok(Some(tag_type_kind));
        }

        Ok(None)
    }

    /// Parses tag types declarations
    /// (e.g. struct Point; )
    pub fn parse_tag_type_declaration(
        &mut self,
        keyword: &KeywordTypes,
    ) -> Result<GlobalNode, String> {
        // qualifiers in a declaration are not used
        self.parse_tag_type_qualifiers()?;

        // move past the tag type keyword
        self.lexer.advance();

        let Some(TokenTypes::Identifier(tag_type_name)) = self.lexer.peek() else {
            unreachable!()
        };

        self.lexer.advance();

        self.parse_tag_type_qualifiers()?;

        let declared_tag_type = GlobalNode::TagType(TagTypeData {
            kind: keyword.into(),
            is_defined: false,
            name: Some(tag_type_name),
            members: vec![],
        });

        self.lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

        Ok(declared_tag_type)
    }

    pub fn get_nested_member_if_some(&mut self) -> Result<Option<Vec<GlobalNode>>, String> {
        let mut items = Vec::new();
        items.extend(match self.is_tag_type_keyword()? {
            Some(value) => match value {
                TagTypeKind::Struct => self.parse_struct_keyword()?,
                TagTypeKind::Union => self.parse_union_keyword()?,
                TagTypeKind::Enum => self.parse_enum_keyword()?,
            },

            None => return Ok(None),
        });

        Ok(Some(items))
    }

    // helpers
    fn token_is_variable_type(token: &TokenTypes) -> bool {
        match token {
            TokenTypes::Identifier(_) => true,
            TokenTypes::Operator(op_type) => match op_type {
                OperatorTypes::LParen | OperatorTypes::Star => true,
                _ => false,
            },

            _ => false,
        }
    }

    fn update_var(&mut self, struct_type: &TypeNode) -> Result<TypeNode, String> {
        let mut var_type = self.parse_type()?;
        var_type.set_most_nested_held_value(struct_type);
        Ok(var_type)
    }

    /// Parses the indentifiers after a variable
    /// This exists because a variable can define multiple different vars using the comma operator
    /// (e.g. int x = 10, y = 20, z;) where the funtion runs after the int x = 10 is parsed
    pub fn parse_vars_after_type<const IS_TAG_TYPE: bool>(
        &mut self,
        original_as_type_node: &TypeNode,
    ) -> Result<Vec<GlobalNode>, String> {
        let mut var_type;
        let mut all_vars = Vec::new();

        for i in 0.. {
            let next_token = self
                .lexer
                .force_peek("Expected end of variable definition, got nothing")?;

            // a struct extra vars don't have to start after a comma, it just starts after the right curly brace
            // or after the qulifiers after the left curly brace
            if IS_TAG_TYPE && i == 0 && Self::token_is_variable_type(&next_token) {
                var_type = self.update_var(&original_as_type_node)?;
            }
            // Could be another variable assigned after the original one
            else if matches!(next_token, TokenTypes::Operator(OperatorTypes::Comma)) {
                self.lexer.advance();
                var_type = self.update_var(&original_as_type_node)?;
            } else if matches!(next_token, TokenTypes::Semicolon) {
                break;
            } else {
                // Variable assignment can only end wiht a comma or semi colon
                return Err(format!(
                    "Expected comma or semicolon after variable definition, got token of type {next_token}"
                ));
            }

            let next_token = self
                .lexer
                .force_peek("Unexpected end of variable definition, got nothing")?;
            let final_var;

            // its a definition
            if matches!(
                next_token,
                TokenTypes::Assignment(AssignmentTypes::SimpleAssignment)
            ) {
                self.lexer.advance();

                final_var = GlobalNode::Initalizer {
                    var_type: var_type.clone(),
                    r_value: Some(self.parse_expression(3)?),
                };
            }
            // its a declaration
            else {
                final_var = GlobalNode::Initalizer {
                    var_type: var_type.clone(),
                    r_value: None,
                };
            }

            all_vars.push(final_var);
        }

        self.lexer.expect(|x| matches!(x, TokenTypes::Semicolon))?;

        Ok(all_vars)
    }

    /// Parses struct or union definitions
    /// These include stuct with a left and right curly brace that may or may not include members
    /// This does not account for variables defined subsequently with the struct
    pub fn parse_struct_or_union_definition<F>(
        &mut self,
        tag_type_kind: TagTypeKind,
        parse_member: F,
    ) -> Result<TagTypeData, String>
    where
        F: Fn(&mut Parser) -> Result<Vec<TagTypeMember>, String>,
    {
        self.lexer.advance(); // move past the struct

        let name = match self.lexer.peek() {
            Some(TokenTypes::Identifier(name)) => {
                self.lexer.advance();
                Some(name)
            }
            _ => None,
        };

        self.lexer.advance();

        let mut members = Vec::new(); // literally everything else including regular struct variables

        while !matches!(self.lexer.peek(), Some(TokenTypes::RCurlyBrace)) {
            let final_member = parse_member(self)?;

            // becuase multiple semicolons are allowed for some reason
            while let Some(TokenTypes::Semicolon) = self.lexer.peek() {
                self.lexer.advance();
            }

            members.extend(final_member);
        }

        self.lexer.advance();

        let final_struct_or_union = TagTypeData {
            kind: tag_type_kind,
            is_defined: true,
            name: name.clone(),
            members,
        };

        Ok(final_struct_or_union)
    }

    pub fn parse_tag_type_definition_and_vars<F>(
        &mut self,
        parse_definition: F,
    ) -> Result<Vec<GlobalNode>, String>
    where
        F: FnOnce(&mut Parser) -> Result<TagTypeData, String>,
    {
        let mut tag_type_and_vars = Vec::new();

        let mut var_qualifiers = self.parse_tag_type_qualifiers()?;

        let mut defined_tag_type = parse_definition(self)?;

        var_qualifiers.extend(self.parse_tag_type_qualifiers()?);

        let generated_new_name = defined_tag_type.name.is_none();
        if generated_new_name {
            defined_tag_type.name = Some(self.semantics.generate_new_name());
        }

        let type_node = defined_tag_type.as_type(&var_qualifiers);

        let defined_vars: Vec<GlobalNode> = self.parse_vars_after_type::<true>(&type_node)?;

        if generated_new_name && defined_vars.is_empty() {
            return Ok(vec![]);
        }

        let storage_class_specifiers: Vec<DataTypes> = var_qualifiers
            .iter()
            .filter(|x| x.is_storage_specifier())
            .map(|x| *x)
            .collect();

        if storage_class_specifiers.len() > 1 {
            return Err(String::from(
                "Disallowed tag type storage class specifier combination",
            ));
        }

        if storage_class_specifiers.contains(&DataTypes::Typedef) {
            for typedef in defined_vars {}

            return Ok(vec![]);
        } else {
            tag_type_and_vars.push(GlobalNode::TagType(defined_tag_type));
            tag_type_and_vars.extend(defined_vars);
        }

        return Ok(tag_type_and_vars);
    }
}
