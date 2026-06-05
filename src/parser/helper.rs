use crate::lexer::lexer::{Lexer, TokenTypes};

pub fn verify_next_in_comma_list(
    lexer: &mut Lexer,
    end_token: TokenTypes,
    error_message: &'static str,
) -> Result<(), String> {
    if let Some(next_token) = lexer.peek()
        && next_token != end_token
    {
        let Some(future_token) = lexer.forward_peek() else {
            return Err(String::from(error_message));
        };

        if future_token != end_token {
            lexer.expect(|x| matches!(x, TokenTypes::Comma))?;
        }
    } else if lexer.peek().is_none() {
        return Err(String::from(error_message));
    }

    return Ok(());
}
