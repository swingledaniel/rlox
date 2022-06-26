use crate::token_type::TokenType;

#[derive(Debug)]
pub enum Literal {
    IdentifierLiteral(String),
    StringLiteral(String),
    F64(f64),
    None,
}

#[derive(Debug)]
pub struct Token {
    pub typ: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: usize,
}
