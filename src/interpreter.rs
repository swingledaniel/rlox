use crate::environment::Environment;
use crate::runtime_error;
use crate::stmt::Stmt;
use crate::token::{Literal::*, Token};
use crate::token_type::TokenType;
use crate::utils::Soo;
use crate::{expr::*, token::Literal};

impl From<&'static str> for Soo {
    fn from(s: &'static str) -> Soo {
        Soo::Static(s)
    }
}

pub fn interpret(statements: Vec<Stmt>, environment: &mut Environment) -> bool {
    for statement in statements.into_iter() {
        match statement.interpret(environment) {
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
    fn interpret(self, environment: &mut Environment) -> Result<Literal, (Token, Soo)>;
}

impl Interpreter for Stmt {
    fn interpret(self, environment: &mut Environment) -> Result<Literal, (Token, Soo)> {
        match self {
            Stmt::Block { statements } => {
                execute_block(statements, environment)?;
            }
            Stmt::Expression { expression } => {
                expression.interpret(environment)?;
            }
            Stmt::Print { expression } => {
                let literal = expression.interpret(environment)?;
                println!("{}", stringify(literal));
            }
            Stmt::Var { name, initializer } => {
                let value = match initializer {
                    Some(expr) => expr.interpret(environment)?,
                    _ => Literal::None,
                };
                environment.define(&name.lexeme, value);
            }
        };
        Ok(Literal::None)
    }
}

impl Interpreter for Expr {
    fn interpret(self, environment: &mut Environment) -> Result<Literal, (Token, Soo)> {
        match self {
            Expr::Assign { name, value } => {
                let literal = value.interpret(environment)?;
                environment.assign(&name, literal)
            }
            Expr::Binary {
                left: left_expr,
                operator,
                right: right_expr,
            } => {
                let left = left_expr.interpret(environment)?;
                let right = right_expr.interpret(environment)?;

                match operator.typ {
                    TokenType::Plus => match (left, right) {
                        (F64(f1), F64(f2)) => Ok(F64(f1 + f2)),
                        (StringLiteral(s1), StringLiteral(s2)) => Ok(StringLiteral(s1 + &s2)),
                        _ => Err((
                            operator,
                            "Operands must be two numbers or two strings.".into(),
                        )),
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
                    _ => Err((operator, "Expected a binary operator.".into())),
                }
            }
            Expr::Grouping { expression } => expression.interpret(environment),
            Expr::LiteralExpr { value } => Ok(value),
            Expr::Unary { operator, right } => {
                let right = right.interpret(environment)?;
                match operator.typ {
                    TokenType::Bang => Ok(Literal::BoolLiteral(!is_truthy(right))),
                    TokenType::Minus => match right {
                        F64(value) => Ok(F64(-value)),
                        _ => Err((operator, "Operand must be a number.".into())),
                    },
                    _ => Err((operator, "Expected a unary operator.".into())),
                }
            }
            Expr::Variable { name } => environment.get(&name),
        }
    }
}

fn execute_block(statements: Vec<Stmt>, environment: &mut Environment) -> Result<(), (Token, Soo)> {
    environment.add_scope();

    for stmt in statements {
        stmt.interpret(environment)?;
    }

    environment.del_scope();
    Ok(())
}

fn get_numeric_operands(
    operator: Token,
    left: Literal,
    right: Literal,
) -> Result<(f64, f64), (Token, Soo)> {
    let message = "Operands must be numbers.";

    let left = match left {
        F64(value) => value,
        _ => return Err((operator, message.into())),
    };
    let right = match right {
        F64(value) => value,
        _ => return Err((operator, message.into())),
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
