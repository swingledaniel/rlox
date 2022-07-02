use crate::expr::Expr;

pub enum Stmt {
    Expression { expression: Box<Expr> },
    Print { expression: Box<Expr> },
}
