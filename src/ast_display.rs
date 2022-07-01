use std::fmt;

use crate::{
    expr::*,
    token::{Literal, Token},
};

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::IdentifierLiteral(identifier) => {
                write!(f, "{}", identifier)
            }
            Literal::StringLiteral(s) => {
                write!(f, "{}", s)
            }
            Literal::BoolLiteral(b) => {
                write!(f, "{}", b)
            }
            Literal::F64(float) => {
                write!(f, "{}", float)
            }
            Literal::None => {
                write!(f, "nil")
            }
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.lexeme)
    }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                write!(f, "({} {} {})", operator, left, right)
            }
            Expr::Grouping { expression } => {
                write!(f, "(group {})", expression)
            }
            Expr::LiteralExpr { value } => {
                write!(f, "{}", value)
            }
            Expr::Unary { operator, right } => {
                write!(f, "({} {})", operator, right)
            }
        }
    }
}
