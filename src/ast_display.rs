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
            Expr::Binary(Binary {
                left,
                operator,
                right,
            }) => {
                write!(f, "({} {} {})", operator, left, right)
            }
            Expr::Grouping(Grouping { expression }) => {
                write!(f, "(group {})", expression)
            }
            Expr::LiteralExpr(LiteralExpr { value }) => {
                write!(f, "{}", value)
            }
            Expr::Unary(Unary { operator, right }) => {
                write!(f, "({} {})", operator, right)
            }
        }
    }
}

impl fmt::Display for Unary {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({} {})", self.operator.lexeme, self.right)
    }
}
