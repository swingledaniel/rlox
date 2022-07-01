use crate::token_type::TokenType;

#[derive(Debug, Clone)]
pub enum Literal {
    IdentifierLiteral(String),
    StringLiteral(String),
    BoolLiteral(bool),
    F64(f64),
    None,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub typ: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: usize,
}
