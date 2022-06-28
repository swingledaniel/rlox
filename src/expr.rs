use crate::token::{Literal, Token};

pub enum Expr {
    Binary {
        left: Box<Expr>,
        operator: Token,
        right: Box<Expr>,
    },
    Grouping {
        expression: Box<Expr>,
    },
    LiteralExpr {
        value: Literal,
    },
    Unary {
        operator: Token,
        right: Box<Expr>,
    },
}
