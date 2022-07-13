use std::time::{SystemTime, UNIX_EPOCH};

use crate::{
    environment::Environment,
    interpreter::execute_block,
    stmt::{self},
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

    pub fn call(self, arguments: Vec<Literal>) -> Result<Literal, (Token, Soo)> {
        match self.kind {
            CallableKind::Function {
                mut declaration,
                mut closure,
            } => {
                closure.add_scope();
                for (param, arg) in self.parameters.iter().zip(arguments.into_iter()) {
                    closure.define(param, arg);
                }

                match execute_block(&mut declaration.body, &mut closure) {
                    Err((token, message)) => {
                        return match (token.typ, token.lexeme.as_str()) {
                            (crate::token_type::TokenType::Return, "RETURN") => Ok(token.literal),
                            _ => Err((token, message)),
                        }
                    }
                    _ => {}
                };
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
