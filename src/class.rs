use std::collections::HashMap;

use crate::callable::Callable;

#[derive(Clone, Debug)]
pub struct Class {
    pub name: String,
    pub methods: HashMap<String, Callable>,
}

impl Class {
    pub fn new(name: String, methods: HashMap<String, Callable>) -> Self {
        Class { name, methods }
    }

    pub fn find_method(&mut self, name: &str) -> Option<Callable> {
        self.methods.get(name).map(|method| method.to_owned())
    }

    pub fn to_string(&self) -> String {
        self.name.to_owned()
    }
}
