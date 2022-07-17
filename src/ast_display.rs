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
            Literal::CallableLiteral(function) => match &function.kind {
                CallableKind::Class(class) => write!(f, "{}", class.to_string()),
                CallableKind::Function {
                    declaration,
                    closure: _,
                } => write!(f, "{}", declaration.name),
                CallableKind::Native(name) => write!(f, "{name}"),
            },
            Literal::F64(float) => {
                write!(f, "{}", float)
            }
            Literal::IdentifierLiteral(identifier) => {
                write!(f, "{}", identifier)
            }
            Literal::InstanceLiteral(instance) => {
                write!(f, "{}", instance.to_string())
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
        match &self.1 {
            ExprKind::Assign { name, value } => {
                write!(f, "{name} = {value}")
            }
            ExprKind::Binary {
                left,
                operator,
                right,
            } => {
                write!(f, "({operator} {left} {right})")
            }
            ExprKind::Call {
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
            ExprKind::Get { object, name } => write!(f, "{object}.{name}"),

            ExprKind::Grouping { expression } => {
                write!(f, "(group {expression})")
            }
            ExprKind::LiteralExpr { value } => {
                write!(f, "{value}")
            }
            ExprKind::Logical {
                left,
                operator,
                right,
            } => {
                write!(f, "{left} {operator} {right}")
            }
            ExprKind::Set {
                object,
                name,
                value,
            } => write!(f, "{object}.{name} = {value}"),
            ExprKind::Unary { operator, right } => {
                write!(f, "({operator} {right})")
            }
            ExprKind::Variable { name } => {
                write!(f, "{name}")
            }
        }
    }
}
