use crate::lexer::language_features::DataTypes;
use crate::lexer::language_features::KeywordTypes;
use crate::lexer::language_features::OperatorTypes;
use crate::lexer::lexer::{Lexer, TokenTypes};
use crate::parser::type_parser::parse_type;
use std::mem;

#[derive(Debug)]
pub enum TagKeywordUsage {
    Definition,
    Declaration,
    Variable,
}

/// Parses the qualifiers of a tag type after its been defined
/// (e.g struct Point{int x;} const volatile p, q;) where the const volatile portion is parsed
pub fn parse_tag_type_qualifiers(lexer: &mut Lexer) -> Result<Vec<DataTypes>, String> {
    let mut qualifiers = Vec::new();

    while let Some(TokenTypes::DataType(data_type)) = lexer.peek() {
        if data_type.is_qualifier() || data_type.is_storage_specifier() {
            qualifiers.push(data_type);
            lexer.advance();
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
pub fn tag_type_keyword_usage(lexer: &mut Lexer) -> Result<TagKeywordUsage, String> {
    lexer.set_flag();

    // the qualifiers don't matter we just want to skip them here
    parse_tag_type_qualifiers(lexer)?;

    lexer.advance(); // move past the tag type keyword

    // Move past the tag type name if it exists
    if let Some(TokenTypes::Identifier(_)) = lexer.peek() {
        lexer.next();
    }

    parse_tag_type_qualifiers(lexer)?;

    let next_token = lexer.force_peek("Expected next token in tag type definition, got nothing")?;

    // make sure we don't mess up the parsing for our parsing functions
    lexer.recede_to_flag();

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
pub fn is_tag_type_keyword(lexer: &mut Lexer, keyword: &KeywordTypes) -> Result<bool, String> {
    let curr_token = lexer.force_peek("Expected next token, got nothing")?;

    if mem::discriminant(&curr_token) == mem::discriminant(&TokenTypes::Keyword(*keyword)) {
        return Ok(true);
    }

    lexer.set_flag();
    let parsed_type = parse_type(lexer)?;
    lexer.recede_to_flag();

    if parsed_type.contains_struct_type() {
        return Ok(true);
    }

    Ok(false)
}
