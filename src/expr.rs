use crate::token::{Literal, Token};

pub enum Expr {
    Binary(Binary),
    Grouping(Grouping),
    LiteralExpr(LiteralExpr),
    Unary(Unary),
}

pub struct Binary {
    pub left: Box<Expr>,
    pub operator: Token,
    pub right: Box<Expr>,
}

pub struct Grouping {
    pub expression: Box<Expr>,
}

pub struct LiteralExpr {
    pub value: Literal,
}

pub struct Unary {
    pub operator: Token,
    pub right: Box<Expr>,
}
