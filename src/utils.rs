use std::fmt;

/// Represents either a static or owned string
pub enum Soo {
    Static(&'static str),
    Owned(String),
}

impl fmt::Display for Soo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Soo::Static(s) => write!(f, "{s}"),
            Soo::Owned(s) => write!(f, "{s}"),
        }
    }
}

impl std::convert::From<&'static str> for Soo {
    fn from(item: &'static str) -> Self {
        Soo::Static(item)
    }
}

impl std::convert::From<String> for Soo {
    fn from(item: String) -> Self {
        Soo::Owned(item)
    }
}

pub struct ExprId(usize);

impl ExprId {
    pub fn new() -> Self {
        ExprId(0)
    }

    pub fn next(&mut self) -> usize {
        let id = self.0;
        self.0 += 1;
        id
    }
}
