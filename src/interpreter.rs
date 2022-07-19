use std::collections::HashMap;

use crate::callable::{Callable, CallableKind};
use crate::environment::Environment;
use crate::runtime_error;
use crate::stmt::Stmt;
use crate::token::{Literal::*, Token};
use crate::token_type::TokenType;
use crate::utils::Soo;
use crate::{expr::*, token::Literal};

pub fn interpret(statements: Vec<Stmt>, environment: &mut Environment) -> bool {
    for mut statement in statements.into_iter() {
        match &mut statement.interpret(environment) {
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
        BoolLiteral(b) => b.to_string(),
        CallableLiteral(function) => match function.kind {
            CallableKind::Class(class) => class.to_string(),
            CallableKind::Function {
                declaration,
                closure: _,
                is_initializer: _,
            } => format!("<fn {}>", declaration.name.lexeme),
            CallableKind::Native(_) => "<native fn>".to_owned(),
        },
        F64(f) => {
            if f.fract() == 0f64 {
                (f as i64).to_string()
            } else {
                f.to_string()
            }
        }
        IdentifierLiteral(ident) => ident,
        InstanceLiteral(instance) => instance.to_string(),
        StringLiteral(s) => s,
        None => "nil".to_owned(),
    }
}

trait Interpreter {
    fn interpret(&mut self, environment: &mut Environment) -> Result<Literal, (Token, Soo)>;
}

impl Interpreter for Stmt {
    fn interpret(&mut self, environment: &mut Environment) -> Result<Literal, (Token, Soo)> {
        match self {
            Stmt::Block { statements } => {
                execute_block(statements, environment)?;
            }
            Stmt::Class {
                name,
                superclass: stmt_superclass,
                methods: stmt_methods,
            } => {
                let superclass = match stmt_superclass {
                    Some(expr) => match expr.interpret(environment)? {
                        CallableLiteral(Callable {
                            arity: _,
                            parameters: _,
                            kind: CallableKind::Class(class),
                        }) => Some(class),
                        _ => {
                            let superclass_name = match &expr.1 {
                                ExprKind::Variable { name } => name,
                                _ => panic!("Superclass was not a variable."),
                            };
                            return Err((
                                superclass_name.clone(),
                                "Superclass must be a class.".into(),
                            ));
                        }
                    },
                    _ => Option::None,
                };

                environment.define(&name.lexeme, Literal::None);

                if let Some(value) = superclass.to_owned() {
                    environment.add_scope();
                    environment.define(
                        "super",
                        CallableLiteral(Callable::new_class(
                            value.name,
                            value.superclass.map(|c| *c),
                            value.methods,
                        )),
                    );
                }

                let mut methods = HashMap::new();
                for method in stmt_methods {
                    let function = Callable::new_function(
                        method,
                        environment.clone(),
                        method.name.lexeme == "init",
                    );
                    methods.insert(method.name.lexeme.to_owned(), function);
                }

                if superclass.is_some() {
                    environment.del_scope();
                }

                let class = Callable::new_class(name.lexeme.to_owned(), superclass, methods);

                environment.assign(name, CallableLiteral(class))?;
            }
            Stmt::Expression { expression } => {
                expression.interpret(environment)?;
            }
            Stmt::Function(stmt) => {
                let function =
                    CallableLiteral(Callable::new_function(stmt, environment.clone(), false));
                environment.define(&stmt.name.lexeme, function);
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if is_truthy(&condition.interpret(environment)?) {
                    then_branch.interpret(environment)?;
                } else if let Some(else_stmt) = else_branch {
                    else_stmt.interpret(environment)?;
                }
            }
            Stmt::Print { expression } => {
                let literal = expression.interpret(environment)?;
                println!("{}", stringify(literal));
            }
            Stmt::Return { keyword, value } => {
                let value = match value {
                    Some(expr) => expr.interpret(environment)?,
                    _ => Literal::None,
                };
                return Err((
                    Token {
                        typ: TokenType::Return,
                        lexeme: "RETURN".to_owned(),
                        literal: value,
                        line: keyword.line,
                    },
                    "".into(),
                ));
            }
            Stmt::Var { name, initializer } => {
                let value = match initializer {
                    Some(expr) => expr.interpret(environment)?,
                    _ => Literal::None,
                };
                environment.define(&name.lexeme, value);
            }
            Stmt::While { condition, body } => {
                while is_truthy(&condition.interpret(environment)?) {
                    body.interpret(environment)?;
                }
            }
        };
        Ok(Literal::None)
    }
}

impl Interpreter for Expr {
    fn interpret(&mut self, environment: &mut Environment) -> Result<Literal, (Token, Soo)> {
        match &mut self.1 {
            ExprKind::Assign { name, value } => {
                let literal = value.interpret(environment)?;

                match environment.locals.get(&self.0) {
                    Some(distance) => environment.assign_at(*distance, &name, literal),
                    _ => environment.assign_global(&name, literal),
                }
            }
            ExprKind::Binary {
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
                            operator.clone(),
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
                    _ => Err((operator.clone(), "Expected a binary operator.".into())),
                }
            }
            ExprKind::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = callee.interpret(environment)?;

                let mut func_args = Vec::new();
                for argument in arguments {
                    func_args.push(argument.interpret(environment)?);
                }

                match callee {
                    CallableLiteral(function) => {
                        if func_args.len() != function.arity {
                            Err((
                                paren.clone(),
                                Soo::Owned(format!(
                                    "Expected {} arguments but got {}.",
                                    function.arity,
                                    func_args.len()
                                )),
                            ))
                        } else {
                            function.call(func_args)
                        }
                    }
                    _ => Err((paren.clone(), "Can only call functions and classes.".into())),
                }
            }
            ExprKind::Get { object, name } => match object.interpret(environment)? {
                InstanceLiteral(mut instance) => instance.get(name),
                _ => Err((name.clone(), "Only instances have properties.".into())),
            },
            ExprKind::Grouping { expression } => expression.interpret(environment),
            ExprKind::LiteralExpr { value } => Ok(value.clone()),
            ExprKind::Logical {
                left,
                operator,
                right,
            } => {
                let left = left.interpret(environment)?;

                match operator.typ {
                    TokenType::Or => {
                        if is_truthy(&left) {
                            return Ok(left);
                        }
                    }
                    _ => {
                        if !is_truthy(&left) {
                            return Ok(left);
                        }
                    }
                };

                right.interpret(environment)
            }
            ExprKind::Set {
                object,
                name,
                value,
            } => match object.interpret(environment)? {
                InstanceLiteral(mut instance) => {
                    let value = value.interpret(environment)?;
                    instance.set(name, value.to_owned());
                    Ok(value)
                }
                _ => Err((name.clone(), "Only instances have fields.".into())),
            },
            ExprKind::Super { keyword: _, method } => {
                let distance = *environment.locals.get(&self.0).unwrap();
                let mut superclass = match environment.get_at(distance, "super").unwrap() {
                    CallableLiteral(Callable {
                        arity: _,
                        parameters: _,
                        kind,
                    }) => match kind {
                        CallableKind::Class(c) => c,
                        _ => panic!("'super' did not resolve to a class."),
                    },
                    _ => panic!("'super' did not resolve to a callable literal."),
                };

                let object = match environment.get_at(distance - 1, "this").unwrap() {
                    InstanceLiteral(instance) => instance,
                    _ => panic!("Subclass did not resolve to an instance."),
                };

                match superclass.find_method(&method.lexeme) {
                    Some(mut method) => {
                        method.bind(object);
                        Ok(CallableLiteral(method))
                    }
                    _ => Err((
                        method.clone(),
                        format!("Undefined property '{}'.", method.lexeme).into(),
                    )),
                }
            }
            ExprKind::This { keyword } => lookup_variable(keyword, self.0, environment),
            ExprKind::Unary { operator, right } => {
                let right = right.interpret(environment)?;
                match operator.typ {
                    TokenType::Bang => Ok(Literal::BoolLiteral(!is_truthy(&right))),
                    TokenType::Minus => match right {
                        F64(value) => Ok(F64(-value)),
                        _ => Err((operator.clone(), "Operand must be a number.".into())),
                    },
                    _ => Err((operator.clone(), "Expected a unary operator.".into())),
                }
            }
            ExprKind::Variable { name } => lookup_variable(&name, self.0, environment),
        }
    }
}

pub fn execute_block(
    statements: &mut Vec<Stmt>,
    environment: &mut Environment,
) -> Result<(), (Token, Soo)> {
    environment.add_scope();

    for stmt in statements {
        stmt.interpret(environment)?;
    }

    environment.del_scope();
    Ok(())
}

pub fn execute_statements(
    statements: &mut Vec<Stmt>,
    environment: &mut Environment,
) -> Result<(), (Token, Soo)> {
    for stmt in statements {
        stmt.interpret(environment)?;
    }
    Ok(())
}

fn get_numeric_operands(
    operator: &mut Token,
    left: Literal,
    right: Literal,
) -> Result<(f64, f64), (Token, Soo)> {
    let message = "Operands must be numbers.";

    let left = match left {
        F64(value) => value,
        _ => return Err((operator.clone(), message.into())),
    };
    let right = match right {
        F64(value) => value,
        _ => return Err((operator.clone(), message.into())),
    };

    Ok((left, right))
}

fn is_truthy(literal: &Literal) -> bool {
    match literal {
        Literal::BoolLiteral(b) => *b,
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

pub fn resolve(id: usize, depth: usize, environment: &mut Environment) {
    environment.locals.insert(id, depth);
}

fn lookup_variable(
    name: &Token,
    id: usize,
    environment: &mut Environment,
) -> Result<Literal, (Token, Soo)> {
    match environment.locals.get(&id) {
        Some(distance) => Ok(environment.get_at(*distance, &name.lexeme).unwrap()),
        _ => match environment
            .layers
            .get(0)
            .unwrap()
            .borrow_mut()
            .get(&name.lexeme)
        {
            Some(var) => Ok(var.to_owned()),
            _ => Err((
                name.clone(),
                format!("Unable to resolve global variable '{}'.", name.lexeme).into(),
            )),
        },
    }
}
