use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    callable::{Callable, CallableKind},
    token::{Literal, Token},
    utils::Soo,
};

#[derive(Clone, Debug)]
pub struct Environment {
    pub layers: Vec<Rc<RefCell<HashMap<String, Literal>>>>,
}

impl Environment {
    pub fn new() -> Self {
        let mut env = Environment {
            layers: vec![Rc::new(RefCell::new(HashMap::new()))],
        };

        // define native functions
        env.define(
            "clock",
            Literal::FunctionLiteral(Callable {
                arity: 0,
                parameters: Vec::new(),
                kind: CallableKind::Native("clock"),
            }),
        );

        env
    }

    pub fn add_scope(&mut self) {
        self.layers.push(Rc::new(RefCell::new(HashMap::new())));
    }

    pub fn del_scope(&mut self) {
        self.layers.pop();
    }

    pub fn define(&mut self, name: &str, value: Literal) {
        self.layers
            .last_mut()
            .unwrap()
            .borrow_mut()
            .insert(name.to_string(), value);
    }

    pub fn get(&self, name: &Token) -> Result<Literal, (Token, Soo)> {
        for values in self.layers.iter().rev() {
            match values.borrow().get(&name.lexeme) {
                Some(literal) => return Ok(literal.to_owned()),
                _ => {}
            };
        }

        Err((
            name.clone(),
            Soo::Owned(format!("Undefined variable '{}'.", name.lexeme)),
        ))
    }

    pub fn assign(&mut self, name: &Token, value: Literal) -> Result<Literal, (Token, Soo)> {
        for values in self.layers.iter_mut().rev() {
            if values.borrow().contains_key(&name.lexeme) {
                values
                    .borrow_mut()
                    .insert(name.lexeme.to_owned(), value.to_owned());
                return Ok(value);
            }
        }

        Err((
            name.clone(),
            Soo::Owned(format!("Undefined variable '{}'.", name.lexeme)),
        ))
    }
}
