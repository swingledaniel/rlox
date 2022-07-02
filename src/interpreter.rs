use crate::runtime_error;
use crate::stmt::Stmt;
use crate::token::{Literal::*, Token};
use crate::token_type::TokenType;
use crate::{expr::*, token::Literal};

pub fn interpret(statements: Vec<Stmt>) -> bool {
    for statement in statements.into_iter() {
        match statement.interpret() {
            Err((token, message)) => {
                runtime_error(token.line, message);
                return true;
            }
            _ => {}
        };
    }
    false
}

fn stringify(literal: Literal) -> String {
    match literal {
        None => "nil".to_owned(),
        IdentifierLiteral(ident) => ident,
        StringLiteral(s) => s,
        BoolLiteral(b) => b.to_string(),
        F64(f) => {
            if f.fract() == 0f64 {
                (f as i64).to_string()
            } else {
                f.to_string()
            }
        }
    }
}

trait Interpreter {
    fn interpret(self) -> Result<Literal, (Token, &'static str)>;
}

impl Interpreter for Stmt {
    fn interpret(self) -> Result<Literal, (Token, &'static str)> {
        match self {
            Stmt::Print { expression } => {
                let literal = expression.interpret()?;
                println!("{}", stringify(literal));
            }
            Stmt::Expression { expression } => {
                expression.interpret()?;
            }
        };
        Ok(Literal::None)
    }
}

impl Interpreter for Expr {
    fn interpret(self) -> Result<Literal, (Token, &'static str)> {
        match self {
            Expr::Binary {
                left,
                operator,
                right,
            } => {
                let left = left.interpret()?;
                let right = right.interpret()?;

                match operator.typ {
                    TokenType::Plus => match (left, right) {
                        (F64(f1), F64(f2)) => Ok(F64(f1 + f2)),
                        (StringLiteral(s1), StringLiteral(s2)) => Ok(StringLiteral(s1 + &s2)),
                        _ => Err((operator, "Operands must be two numbers or two strings.")),
                    },
                    TokenType::Minus => {
                        let (left, right) = get_numeric_operands(operator, left, right)?;
                        Ok(F64(left - right))
                    }
                    TokenType::Slash => {
                        let (left, right) = get_numeric_operands(operator, left, right)?;
                        Ok(F64(left / right))
                    }
                    TokenType::Star => {
                        let (left, right) = get_numeric_operands(operator, left, right)?;
                        Ok(F64(left * right))
                    }
                    TokenType::Greater => {
                        let (left, right) = get_numeric_operands(operator, left, right)?;
                        Ok(BoolLiteral(left > right))
                    }
                    TokenType::GreaterEqual => {
                        let (left, right) = get_numeric_operands(operator, left, right)?;
                        Ok(BoolLiteral(left >= right))
                    }
                    TokenType::Less => {
                        let (left, right) = get_numeric_operands(operator, left, right)?;
                        Ok(BoolLiteral(left < right))
                    }
                    TokenType::LessEqual => {
                        let (left, right) = get_numeric_operands(operator, left, right)?;
                        Ok(BoolLiteral(left <= right))
                    }
                    TokenType::BangEqual => Ok(BoolLiteral(!is_equal(left, right))),
                    TokenType::EqualEqual => Ok(BoolLiteral(is_equal(left, right))),
                    _ => Err((operator, "Expected a binary operator.")),
                }
            }
            Expr::Grouping { expression } => expression.interpret(),
            Expr::LiteralExpr { value } => Ok(value),
            Expr::Unary { operator, right } => {
                let right = right.interpret()?;
                match operator.typ {
                    TokenType::Bang => Ok(Literal::BoolLiteral(!is_truthy(right))),
                    TokenType::Minus => match right {
                        F64(value) => Ok(F64(-value)),
                        _ => Err((operator, "Operand must be a number.")),
                    },
                    _ => Err((operator, "Expected a unary operator.")),
                }
            }
        }
    }
}

fn get_numeric_operands(
    operator: Token,
    left: Literal,
    right: Literal,
) -> Result<(f64, f64), (Token, &'static str)> {
    let message = "Operands must be numbers.";

    let left = match left {
        F64(value) => value,
        _ => return Err((operator, message)),
    };
    let right = match right {
        F64(value) => value,
        _ => return Err((operator, message)),
    };

    Ok((left, right))
}

fn is_truthy(literal: Literal) -> bool {
    match literal {
        Literal::BoolLiteral(b) => b,
        Literal::None => false,
        _ => true,
    }
}

fn is_equal(left: Literal, right: Literal) -> bool {
    match (left, right) {
        (None, None) => true,
        (None, _) => false,
        (BoolLiteral(b1), BoolLiteral(b2)) => b1 == b2,
        (F64(f1), F64(f2)) => f1 == f2,
        (IdentifierLiteral(ident1), IdentifierLiteral(ident2)) => ident1 == ident2,
        (StringLiteral(s1), StringLiteral(s2)) => s1 == s2,
        _ => false,
    }
}
