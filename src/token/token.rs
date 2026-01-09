#[derive(Debug)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: String,
}

impl Token {
    pub fn new(token_type: TokenType, literal: String) -> Self {
        Token {
            token_type,
            literal,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Illegal,
    EOF,
    Comment,

    // Identifiers + literals
    Identifier,
    Integer,
    Float,
    String,
    Symbol,
    SymbolKey,

    // String interpolation
    InterpolStart,
    InterpolEnd,

    // Operators
    Assign,
    Plus,
    Minus,
    Asterisk,
    Slash,
    Power,
    Eq,
    NotEq,
    LT,
    GT,
    LTE,
    GTE,
    And,
    Or,
    Bang,
    Dot,
    HashRocket,

    // Delimiters
    Comma,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBrack,
    RBrack,
    Semicolon,

    // Keywords
    Def,
    Do,
    End,
    If,
    Else,
    Elsif,
    While,
    Return,
    True,
    False,
    Nil,
}

pub fn lookup_identifier(ident: &str) -> TokenType {
    match ident {
        "def" => TokenType::Def,
        "do" => TokenType::Do,
        "end" => TokenType::End,
        "if" => TokenType::If,
        "else" => TokenType::Else,
        "elsif" => TokenType::Elsif,
        "while" => TokenType::While,
        "return" => TokenType::Return,
        "true" => TokenType::True,
        "false" => TokenType::False,
        "nil" => TokenType::Nil,
        _ => TokenType::Identifier,
    }
}
