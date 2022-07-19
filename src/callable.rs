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
        is_initializer: bool,
    },
    Native(&'static str),
}

impl Callable {
    pub fn new_function(
        declaration: &mut stmt::Function,
        closure: Environment,
        is_initializer: bool,
    ) -> Self {
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
                is_initializer,
            },
        }
    }

    pub fn new_class(
        name: String,
        superclass: Option<crate::class::Class>,
        methods: HashMap<String, Callable>,
    ) -> Self {
        Callable {
            arity: methods.get("init").map(|f| f.arity).unwrap_or(0),
            parameters: Vec::new(),
            kind: CallableKind::Class(Class::new(name, superclass, methods)),
        }
    }

    pub fn call(self, arguments: Vec<Literal>) -> Result<Literal, (Token, Soo)> {
        match self.kind {
            CallableKind::Class(class) => {
                let mut instance = Instance::new(class);
                if let Some(mut initializer) = instance.class.find_method("init") {
                    initializer.bind(instance.clone());
                    initializer.call(arguments)?;
                }

                Ok(Literal::InstanceLiteral(instance))
            }
            CallableKind::Function {
                mut declaration,
                mut closure,
                is_initializer,
            } => {
                closure.add_scope();
                for (param, arg) in self.parameters.iter().zip(arguments.into_iter()) {
                    closure.define(param, arg);
                }

                match execute_statements(&mut declaration.body, &mut closure) {
                    Err((token, message)) => {
                        return match (token.typ, token.lexeme.as_str()) {
                            (crate::token_type::TokenType::Return, "RETURN") => {
                                if is_initializer {
                                    Ok(closure.get_at(0, "this").unwrap())
                                } else {
                                    Ok(token.literal)
                                }
                            }
                            _ => Err((token, message)),
                        }
                    }
                    _ => {}
                };

                closure.del_scope();

                if is_initializer {
                    Ok(closure.get_at(0, "this").unwrap())
                } else {
                    Ok(Literal::None)
                }
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

    pub fn bind(&mut self, instance: Instance) {
        match &mut self.kind {
            CallableKind::Function {
                declaration: _,
                closure,
                is_initializer: _,
            } => {
                closure.add_scope();
                closure.define("this", Literal::InstanceLiteral(instance));
            }
            _ => panic!("Bind called for class or native function"),
        }
    }
}
