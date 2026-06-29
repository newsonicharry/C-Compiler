use crate::lexer::language_features::KeywordTypes;
use crate::lexer::lexer::{Lexer, TokenTypes};

use std::collections::HashMap;

#[allow(dead_code)]
struct TypedefCollection {
    typedefs: HashMap<String, i32>,
}

fn skip_block(lexer: &mut Lexer) {
    while let Some(token) = lexer.next()
        && !matches!(token, TokenTypes::RCurlyBrace)
    {}
}

// a typedef can be anywhere in the current line, so we check
// this doesnt work with typedef after the struct initalizer
pub fn is_typedef(lexer: &mut Lexer) -> bool {
    let mut found_typedef = false;

    // mark where we currently are at
    lexer.set_flag();

    while let Some(token) = lexer.next()
        && !matches!(token, TokenTypes::Semicolon)
    {
        // if there is a struct we don't want prematurely stop within it
        if matches!(token, TokenTypes::LCurlyBrace) {
            skip_block(lexer);
        }

        if matches!(token, TokenTypes::Keyword(KeywordTypes::Typedef)) {
            found_typedef = true;
            break;
        }
    }

    // go to our original location
    lexer.recede_to_flag();

    found_typedef
}
