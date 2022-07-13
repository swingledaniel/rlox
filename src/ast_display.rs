use std::fmt;

use crate::{
    callable::CallableKind,
    expr::*,
    token::{Literal, Token},
};

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::BoolLiteral(b) => {
                write!(f, "{}", b)
            }
            Literal::FunctionLiteral(function) => match function.kind {
                CallableKind::Function(_) => write!(f, "function"),
                CallableKind::Native(name) => write!(f, "{name}"),
            },
            Literal::F64(float) => {
                write!(f, "{}", float)
            }
            Literal::IdentifierLiteral(identifier) => {
                write!(f, "{}", identifier)
            }
            Literal::StringLiteral(s) => {
                write!(f, "{}", s)
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
            Expr::Assign { name, value } => {
                write!(f, "{name} = {value}")
            }
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                write!(f, "({operator} {left} {right})")
            }
            Expr::Call {
                callee,
                paren: _,
                arguments,
            } => {
                write!(f, "{callee}(")?;
                match arguments.get(0) {
                    Some(expr) => write!(f, "{expr}")?,
                    _ => {}
                }
                arguments.iter().skip(1).fold(Ok(()), |result, expr| {
                    result.and_then(|_| write!(f, ",{expr}"))
                })?;
                write!(f, ")")
            }
            Expr::Grouping { expression } => {
                write!(f, "(group {expression})")
            }
            Expr::LiteralExpr { value } => {
                write!(f, "{value}")
            }
            Expr::Logical {
                left,
                operator,
                right,
            } => {
                write!(f, "{left} {operator} {right}")
            }
            Expr::Unary { operator, right } => {
                write!(f, "({operator} {right})")
            }
            Expr::Variable { name } => {
                write!(f, "{name}")
            }
        }
    }
}
