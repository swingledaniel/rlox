use std::collections::HashMap;

use crate::{
    environment::Environment,
    error,
    expr::{Expr, ExprKind},
    stmt::{Function, Stmt},
    token::Token,
    utils::Soo,
};

#[derive(Eq, PartialEq)]
pub enum FunctionType {
    Function,
    Initializer,
    Method,
}

pub enum ClassType {
    Class,
    Subclass,
}

trait Resolver {
    fn resolve(
        &mut self,
        environment: &mut Environment,
        function_stack: &mut Vec<FunctionType>,
        class_stack: &mut Vec<ClassType>,
        had_error: &mut bool,
    ) -> Result<(), (Token, Soo)>;
}

impl Resolver for Stmt {
    fn resolve(
        &mut self,
        environment: &mut Environment,
        function_stack: &mut Vec<FunctionType>,
        class_stack: &mut Vec<ClassType>,
        had_error: &mut bool,
    ) -> Result<(), (Token, Soo)> {
        match self {
            Stmt::Block { statements } => {
                begin_scope(environment);
                resolve_statements(
                    statements,
                    environment,
                    function_stack,
                    class_stack,
                    had_error,
                )?;
                end_scope(environment);
                Ok(())
            }
            Stmt::Class {
                name,
                superclass,
                methods,
            } => {
                class_stack.push(ClassType::Class);

                declare(name, environment, had_error);
                define(name, environment);

                if let Some(expr) = superclass {
                    class_stack.pop();
                    class_stack.push(ClassType::Subclass);

                    let exprkind = &expr.1;
                    match exprkind {
                        ExprKind::Variable {
                            name: superclass_name,
                        } => {
                            if name.lexeme == superclass_name.lexeme {
                                error(name.line, &("A class can't inherit from itself.".into()));
                                *had_error = true;
                            }
                        }
                        _ => panic!("Superclass was not a variable"),
                    }

                    expr.resolve(environment, function_stack, class_stack, had_error)?;

                    begin_scope(environment);
                    environment
                        .scopes
                        .last_mut()
                        .unwrap()
                        .insert("super".to_string(), true);
                }

                begin_scope(environment);
                environment
                    .scopes
                    .last_mut()
                    .unwrap()
                    .insert("this".to_owned(), true);

                for method in methods {
                    let declaration = if method.name.lexeme == "init" {
                        FunctionType::Initializer
                    } else {
                        FunctionType::Method
                    };
                    function_stack.push(declaration);
                    resolve_function(method, environment, function_stack, class_stack, had_error)?;
                    function_stack.pop();
                }

                end_scope(environment);

                if superclass.is_some() {
                    end_scope(environment);
                }

                class_stack.pop();
                Ok(())
            }
            Stmt::Expression { expression } => {
                expression.resolve(environment, function_stack, class_stack, had_error)
            }
            Stmt::Function(function) => {
                declare(&mut function.name, environment, had_error);
                define(&mut function.name, environment);

                function_stack.push(FunctionType::Function);
                resolve_function(
                    function,
                    environment,
                    function_stack,
                    class_stack,
                    had_error,
                )?;
                function_stack.pop();

                Ok(())
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                condition.resolve(environment, function_stack, class_stack, had_error)?;
                then_branch.resolve(environment, function_stack, class_stack, had_error)?;
                if let Some(stmt) = else_branch {
                    stmt.resolve(environment, function_stack, class_stack, had_error)?;
                }
                Ok(())
            }
            Stmt::Print { expression } => {
                expression.resolve(environment, function_stack, class_stack, had_error)
            }
            Stmt::Return { keyword, value } => {
                if function_stack.is_empty() {
                    error(keyword.line, &("Can't return from top-level code.".into()));
                    *had_error = true;
                }

                if let Some(expr) = value {
                    if function_stack.last().is_some_and(|&current_function| {
                        *current_function == FunctionType::Initializer
                    }) {
                        error(
                            keyword.line,
                            &("Can't return a value from an initializer.".into()),
                        );
                        *had_error = true;
                    }

                    expr.resolve(environment, function_stack, class_stack, had_error)?;
                }
                Ok(())
            }
            Stmt::Var { name, initializer } => {
                declare(name, environment, had_error);
                if let Some(expr) = initializer {
                    expr.resolve(environment, function_stack, class_stack, had_error)?;
                }
                define(name, environment);
                Ok(())
            }
            Stmt::While { condition, body } => {
                condition.resolve(environment, function_stack, class_stack, had_error)?;
                body.resolve(environment, function_stack, class_stack, had_error)
            }
        }
    }
}

impl Resolver for Expr {
    fn resolve(
        &mut self,
        environment: &mut Environment,
        function_stack: &mut Vec<FunctionType>,
        class_stack: &mut Vec<ClassType>,
        had_error: &mut bool,
    ) -> Result<(), (Token, Soo)> {
        match &mut self.1 {
            ExprKind::Assign { name, value } => {
                value.resolve(environment, function_stack, class_stack, had_error)?;
                let name = name.clone();
                resolve_local(self.0, &name, environment)
            },
            ExprKind::Binary {
                left,
                operator: _,
                right,
            } => {
                left.resolve(environment, function_stack, class_stack, had_error)?;
                right.resolve(environment, function_stack, class_stack, had_error)
            },
            ExprKind::Call {
                callee,
                paren: _,
                arguments,
            } => {
                callee.resolve(environment, function_stack, class_stack, had_error)?;

                for argument in arguments {
                    argument.resolve(environment, function_stack, class_stack, had_error)?;
                }

                Ok(())
            },
            ExprKind::Get { object, name: _ } => object.resolve(environment, function_stack, class_stack, had_error),
            ExprKind::Grouping { expression } => expression.resolve(environment, function_stack, class_stack, had_error),
            ExprKind::LiteralExpr { value: _ } => Ok(()),
            ExprKind::Logical {
                left,
                operator: _,
                right,
            } => {
                left.resolve(environment, function_stack, class_stack, had_error)?;
                right.resolve(environment, function_stack, class_stack, had_error)
            },
            ExprKind::Set { object, name: _, value } => {
                value.resolve(environment, function_stack, class_stack, had_error)?;
                object.resolve(environment, function_stack, class_stack, had_error)
            }
            ExprKind::Super { keyword, method: _ } => {
                match class_stack.last() {
                    None => {
                        error(keyword.line, &("Can't use 'super' outside of a class.".into()));
                        *had_error = true;
                    }
                    Some(ClassType::Class) => {
                        error(keyword.line, &("Can't use 'super' in a class with no superclass.".into()));
                        *had_error = true;
                    }
                    _ => {}
                }

                resolve_local(self.0, keyword, environment)
            }
            ExprKind::This { keyword } => {
                if class_stack.is_empty() {
                    error(keyword.line, &("Can't use 'this' outside of a class.".into()));
                    *had_error = true;
                    Ok(())
                }
                 else {resolve_local(self.0, keyword, environment)}},
            ExprKind::Unary { operator: _, right } => right.resolve(environment, function_stack, class_stack, had_error),
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
    function_stack: &mut Vec<FunctionType>,
    class_stack: &mut Vec<ClassType>,
    had_error: &mut bool,
) -> Result<(), (Token, Soo)> {
    begin_scope(environment);
    for param in &mut function.params {
        declare(param, environment, had_error);
        define(param, environment);
    }
    resolve_statements(
        &mut function.body,
        environment,
        function_stack,
        class_stack,
        had_error,
    )?;
    end_scope(environment);
    Ok(())
}

pub fn resolve_statements(
    statements: &mut [Stmt],
    environment: &mut Environment,
    function_stack: &mut Vec<FunctionType>,
    class_stack: &mut Vec<ClassType>,
    had_error: &mut bool,
) -> Result<(), (Token, Soo)> {
    for statement in statements {
        statement.resolve(environment, function_stack, class_stack, had_error)?;
    }
    Ok(())
}
