use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{
    class::Class,
    token::{Literal, Token},
    utils::Soo,
};

#[derive(Debug)]
pub struct Instance {
    pub class: Class,
    fields: Rc<RefCell<HashMap<String, Literal>>>,
}

impl Instance {
    pub fn new(class: Class) -> Self {
        Instance {
            class,
            fields: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn get(&mut self, name: &Token) -> Result<Literal, (Token, Soo)> {
        match self.fields.borrow_mut().get(&name.lexeme) {
            Some(value) => Ok(value.clone()),
            _ => match self.class.find_method(&name.lexeme) {
                Some(mut method) => {
                    method.bind(self.clone());
                    Ok(Literal::CallableLiteral(method))
                }
                _ => Err((
                    name.clone(),
                    format!("Undefined property '{}'.", name.lexeme).into(),
                )),
            },
        }
    }

    pub fn set(&mut self, name: &Token, value: Literal) {
        self.fields
            .borrow_mut()
            .insert(name.lexeme.to_owned(), value);
    }

    pub fn to_string(&self) -> String {
        self.class.to_string() + " instance"
    }
}

impl Clone for Instance {
    fn clone(&self) -> Self {
        Instance {
            class: self.class.clone(),
            fields: Rc::clone(&self.fields),
        }
    }
}
