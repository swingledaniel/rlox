use crate::callable::Callable;
use crate::token_type::TokenType;

#[derive(Debug, Clone)]
pub enum Literal {
    BoolLiteral(bool),
    FunctionLiteral(Callable),
    F64(f64),
    IdentifierLiteral(String),
    StringLiteral(String),
    None,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub typ: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: usize,
}
