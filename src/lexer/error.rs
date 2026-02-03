use crate::token::Token;

pub enum LexerErrorType {
    UnexpectedCharacter,
    UnterminatedString,
    InvalidNumber,
    InvalidSymbol,
}

pub struct LexerError {
    pub token: Token,
    pub error: LexerErrorType,
}

impl LexerError {
    pub fn new(token: Token, error: LexerErrorType) -> Self {
        LexerError { token, error }
    }

    pub fn message(&self, source: &str) -> String {
        match self.error {
            LexerErrorType::UnexpectedCharacter => {
                format!("Unexpected character: {:?}", self.token.lexeme(source))
            }
            LexerErrorType::InvalidNumber => {
                format!("Invalid number format: {:?}", self.token.lexeme(source))
            }
            LexerErrorType::UnterminatedString => {
                format!("Unterminated string: {:?}", self.token.lexeme(source))
            }
            LexerErrorType::InvalidSymbol => {
                format!("Invalid symbol format: {:?}", self.token.lexeme(source))
            }
        }
    }
}
