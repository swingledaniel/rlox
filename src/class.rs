use std::collections::HashMap;

use crate::callable::Callable;

#[derive(Clone, Debug)]
pub struct Class {
    pub name: String,
    pub superclass: Option<Box<Class>>,
    pub methods: HashMap<String, Callable>,
}

impl Class {
    pub fn new(
        name: String,
        superclass: Option<Class>,
        methods: HashMap<String, Callable>,
    ) -> Self {
        Class {
            name,
            superclass: superclass.map(|c| Box::new(c)),
            methods,
        }
    }

    pub fn find_method(&mut self, name: &str) -> Option<Callable> {
        let mut method = self.methods.get(name).map(|method| method.to_owned());
        if method.is_none() {
            if let Some(superclass) = &mut self.superclass {
                method = superclass.find_method(name);
            }
        }
        method
    }

    pub fn to_string(&self) -> String {
        self.name.to_owned()
    }
}
