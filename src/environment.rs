use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    callable::{Callable, CallableKind},
    token::{Literal, Token},
    utils::Soo,
};

#[derive(Clone, Debug)]
pub struct Environment {
    pub layers: Vec<Rc<RefCell<HashMap<String, Literal>>>>,
    pub scopes: Vec<HashMap<String, bool>>,
    pub locals: HashMap<usize, usize>,
}

impl Environment {
    pub fn new() -> Self {
        let mut env = Environment {
            layers: vec![Rc::new(RefCell::new(HashMap::new()))],
            scopes: Vec::new(),
            locals: HashMap::new(),
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

    pub fn ancestor(&mut self, distance: usize) -> Rc<RefCell<HashMap<String, Literal>>> {
        Rc::clone(self.layers.get(self.layers.len() - distance - 1).unwrap())
    }

    pub fn get_at(&mut self, distance: usize, name: &str) -> Option<Literal> {
        self.ancestor(distance)
            .borrow_mut()
            .get(name)
            .map(|literal| literal.to_owned())
    }

    pub fn assign_at(
        &mut self,
        distance: usize,
        name: &Token,
        value: Literal,
    ) -> Result<Literal, (Token, Soo)> {
        self.ancestor(distance)
            .borrow_mut()
            .insert(name.lexeme.to_owned(), value.to_owned());
        Ok(value)
    }

    pub fn assign_global(&mut self, name: &Token, value: Literal) -> Result<Literal, (Token, Soo)> {
        let global = self.layers.get(0).unwrap();
        if global.borrow().contains_key(&name.lexeme) {
            global
                .borrow_mut()
                .insert(name.lexeme.to_owned(), value.to_owned());
            Ok(value)
        } else {
            Err((
                name.clone(),
                Soo::Owned(format!("Undefined variable '{}'.", name.lexeme)),
            ))
        }
    }
}
