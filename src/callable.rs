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
    Function(Box<stmt::Function>),
    Native(&'static str),
}

impl Callable {
    pub fn new_function(declaration: &mut stmt::Function) -> Self {
        Callable {
            arity: declaration.params.len(),
            parameters: declaration
                .params
                .iter()
                .map(|token| token.lexeme.to_owned())
                .collect(),
            kind: CallableKind::Function(Box::new(declaration.clone())),
        }
    }

    pub fn call(
        self,
        environment: &mut Environment,
        arguments: Vec<Literal>,
    ) -> Result<Literal, (Token, Soo)> {
        match self.kind {
            CallableKind::Function(mut declaration) => {
                let mut function_env = Environment {
                    layers: vec![environment.layers.first().unwrap().clone()],
                };
                function_env.add_scope();
                for (param, arg) in self.parameters.iter().zip(arguments.into_iter()) {
                    function_env.define(param, arg);
                }

                execute_block(&mut declaration.body, &mut function_env)?;
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
