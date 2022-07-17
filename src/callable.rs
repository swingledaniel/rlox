use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    class::Class,
    environment::Environment,
    instance::Instance,
    interpreter::execute_statements,
    stmt,
    token::{Literal, Token},
    utils::Soo,
};

#[derive(Clone, Debug)]
pub struct Callable {
    pub arity: usize,
    pub parameters: Vec<String>,
    pub kind: CallableKind,
}

#[derive(Clone, Debug)]
pub enum CallableKind {
    Class(crate::class::Class),
    Function {
        declaration: Box<stmt::Function>,
        closure: Environment,
    },
    Native(&'static str),
}

impl Callable {
    pub fn new_function(declaration: &mut stmt::Function, closure: Environment) -> Self {
        Callable {
            arity: declaration.params.len(),
            parameters: declaration
                .params
                .iter()
                .map(|token| token.lexeme.to_owned())
                .collect(),
            kind: CallableKind::Function {
                declaration: Box::new(declaration.clone()),
                closure,
            },
        }
    }

    pub fn new_class(name: String, methods: HashMap<String, Callable>) -> Self {
        Callable {
            arity: 0,
            parameters: Vec::new(),
            kind: CallableKind::Class(Class::new(name, methods)),
        }
    }

    pub fn call(self, arguments: Vec<Literal>) -> Result<Literal, (Token, Soo)> {
        match self.kind {
            CallableKind::Class(class) => Ok(Literal::InstanceLiteral(Instance::new(class))),
            CallableKind::Function {
                mut declaration,
                mut closure,
            } => {
                closure.add_scope();
                for (param, arg) in self.parameters.iter().zip(arguments.into_iter()) {
                    closure.define(param, arg);
                }

                match execute_statements(&mut declaration.body, &mut closure) {
                    Err((token, message)) => {
                        return match (token.typ, token.lexeme.as_str()) {
                            (crate::token_type::TokenType::Return, "RETURN") => Ok(token.literal),
                            _ => Err((token, message)),
                        }
                    }
                    _ => {}
                };

                closure.del_scope();
                Ok(Literal::None)
            }
            CallableKind::Native(name) => match name {
                "clock" => Ok(Literal::F64(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as f64
                        / 1000.0,
                )),
                _ => unimplemented!(),
            },
        }
    }
}
