use std::collections::HashMap;

use crate::{
    environment::Environment,
    error,
    expr::{Expr, ExprKind},
    stmt::{Function, Stmt},
    token::Token,
    utils::Soo,
};

pub enum FunctionType {
    Function,
}

trait Resolver {
    fn resolve(
        &mut self,
        environment: &mut Environment,
        type_stack: &mut Vec<FunctionType>,
        had_error: &mut bool,
    ) -> Result<(), (Token, Soo)>;
}

impl Resolver for Stmt {
    fn resolve(
        &mut self,
        environment: &mut Environment,
        type_stack: &mut Vec<FunctionType>,
        had_error: &mut bool,
    ) -> Result<(), (Token, Soo)> {
        match self {
            Stmt::Block { statements } => {
                begin_scope(environment);
                resolve_statements(statements, environment, type_stack, had_error)?;
                end_scope(environment);
                Ok(())
            }
            Stmt::Expression { expression } => {
                expression.resolve(environment, type_stack, had_error)
            }
            Stmt::Function(function) => {
                declare(&mut function.name, environment, had_error);
                define(&mut function.name, environment);

                type_stack.push(FunctionType::Function);
                resolve_function(function, environment, type_stack, had_error)?;
                type_stack.pop();

                Ok(())
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                condition.resolve(environment, type_stack, had_error)?;
                then_branch.resolve(environment, type_stack, had_error)?;
                if let Some(stmt) = else_branch {
                    stmt.resolve(environment, type_stack, had_error)?;
                }
                Ok(())
            }
            Stmt::Print { expression } => expression.resolve(environment, type_stack, had_error),
            Stmt::Return { keyword, value } => {
                if type_stack.is_empty() {
                    error(keyword.line, &("Can't return from top-level code.".into()));
                    *had_error = true;
                }

                if let Some(expr) = value {
                    expr.resolve(environment, type_stack, had_error)?;
                }
                Ok(())
            }
            Stmt::Var { name, initializer } => {
                declare(name, environment, had_error);
                if let Some(expr) = initializer {
                    expr.resolve(environment, type_stack, had_error)?;
                }
                define(name, environment);
                Ok(())
            }
            Stmt::While { condition, body } => {
                condition.resolve(environment, type_stack, had_error)?;
                body.resolve(environment, type_stack, had_error)
            }
        }
    }
}

impl Resolver for Expr {
    fn resolve(
        &mut self,
        environment: &mut Environment,
        type_stack: &mut Vec<FunctionType>,
        had_error: &mut bool,
    ) -> Result<(), (Token, Soo)> {
        match &mut self.1 {
            ExprKind::Assign { name, value } => {
                value.resolve(environment, type_stack, had_error)?;
                let name = name.clone();
                resolve_local(self.0, &name, environment)
            },
            ExprKind::Binary {
                left,
                operator: _,
                right,
            } => {
                left.resolve(environment, type_stack, had_error)?;
                right.resolve(environment, type_stack, had_error)
            },
            ExprKind::Call {
                callee,
                paren: _,
                arguments,
            } => {
                callee.resolve(environment, type_stack, had_error)?;

                for argument in arguments {
                    argument.resolve(environment, type_stack, had_error)?;
                }

                Ok(())
            },
            ExprKind::Grouping { expression } => expression.resolve(environment, type_stack, had_error),
            ExprKind::LiteralExpr { value: _ } => Ok(()),
            ExprKind::Logical {
                left,
                operator: _,
                right,
            } => {
                left.resolve(environment, type_stack, had_error)?;
                right.resolve(environment, type_stack, had_error)
            },
            ExprKind::Unary { operator: _, right } => right.resolve(environment, type_stack, had_error),
            ExprKind::Variable { name } => {
                if let Some(scope) = environment.scopes.last_mut() && scope.get(&name.lexeme).is_some_and(|&&b| !b) {
                    Err((name.clone(), "Can't read local variable in its own initializer.".into()))
                } else {
                    let name = name.clone();
                    resolve_local(self.0, &name, environment)
                }
            }
        }
    }
}

fn begin_scope(environment: &mut Environment) {
    environment.scopes.push(HashMap::new());
}

fn end_scope(environment: &mut Environment) {
    environment.scopes.pop();
}

fn declare(name: &mut Token, environment: &mut Environment, had_error: &mut bool) {
    if let Some(scope) = environment.scopes.last_mut() {
        if scope.contains_key(&name.lexeme) {
            error(
                name.line,
                &("Already a variable with this name in this scope.".into()),
            );
            *had_error = true;
        }
        scope.insert(name.lexeme.clone(), false);
    }
}

fn define(name: &mut Token, environment: &mut Environment) {
    if let Some(scope) = environment.scopes.last_mut() {
        scope.insert(name.lexeme.clone(), true);
    }
}

fn resolve_local(
    id: usize,
    name: &Token,
    environment: &mut Environment,
) -> Result<(), (Token, Soo)> {
    for (i, scope) in environment.scopes.iter_mut().rev().enumerate() {
        if scope.contains_key(&name.lexeme) {
            crate::interpreter::resolve(id, i, environment);
            break;
        }
    }
    Ok(())
}

fn resolve_function(
    function: &mut Function,
    environment: &mut Environment,
    type_stack: &mut Vec<FunctionType>,
    had_error: &mut bool,
) -> Result<(), (Token, Soo)> {
    begin_scope(environment);
    for param in &mut function.params {
        declare(param, environment, had_error);
        define(param, environment);
    }
    resolve_statements(&mut function.body, environment, type_stack, had_error)?;
    end_scope(environment);
    Ok(())
}

pub fn resolve_statements(
    statements: &mut [Stmt],
    environment: &mut Environment,
    type_stack: &mut Vec<FunctionType>,
    had_error: &mut bool,
) -> Result<(), (Token, Soo)> {
    for statement in statements {
        statement.resolve(environment, type_stack, had_error)?;
    }
    Ok(())
}
