use crate::callable::Callable;
use crate::instance::Instance;
use crate::token_type::TokenType;

#[derive(Clone, Debug)]
pub enum Literal {
    BoolLiteral(bool),
    CallableLiteral(Callable),
    F64(f64),
    IdentifierLiteral(String),
    InstanceLiteral(Instance),
    StringLiteral(String),
    None,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub typ: TokenType,
    pub lexeme: String,
    pub literal: Literal,
    pub line: usize,
}
