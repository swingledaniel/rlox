use std::collections::HashMap;

use crate::{
    token::{Literal, Token},
    utils::Soo,
};

pub struct Environment {
    layers: Vec<HashMap<String, Literal>>,
}

impl Environment {
    pub fn new() -> Self {
        Environment {
            layers: vec![HashMap::new()],
        }
    }

    pub fn add_scope(&mut self) {
        self.layers.push(HashMap::new());
    }

    pub fn del_scope(&mut self) {
        self.layers.pop();
    }

    pub fn define(&mut self, name: &str, value: Literal) {
        self.layers
            .last_mut()
            .unwrap()
            .insert(name.to_string(), value);
    }

    pub fn get(&self, name: &Token) -> Result<Literal, (Token, Soo)> {
        for values in self.layers.iter().rev() {
            match values.get(&name.lexeme) {
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
            if values.contains_key(&name.lexeme) {
                values.insert(name.lexeme.to_owned(), value.to_owned());
                return Ok(value);
            }
        }

        Err((
            name.clone(),
            Soo::Owned(format!("Undefined variable '{}'.", name.lexeme)),
        ))
    }
}
