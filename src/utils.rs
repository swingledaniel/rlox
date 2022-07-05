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
