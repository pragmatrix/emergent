use crate::Scoped;
use std::borrow::Cow;
use std::fmt;

pub type ScopePath = Vec<Scope>;

impl Scoped for Vec<Scope> {
    fn scoped(mut self, scope: impl Into<Scope>) -> Self {
        // TODO: rather unfortunate that we need to prepend here. May use another datatype.
        self.insert(0, scope.into());
        self
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum Scope {
    Name(Cow<'static, str>),
    Index(usize),
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Scope::*;
        match self {
            Name(n) => write!(f, "'{}'", n.to_owned()),
            Index(i) => write!(f, "{}", i),
        }
    }
}

impl From<&'static str> for Scope {
    fn from(str: &'static str) -> Self {
        Scope::Name(Cow::from(str))
    }
}

impl From<String> for Scope {
    fn from(str: String) -> Self {
        Scope::Name(Cow::from(str))
    }
}

impl From<&String> for Scope {
    fn from(str: &String) -> Self {
        str.clone().into()
    }
}

impl From<usize> for Scope {
    fn from(i: usize) -> Self {
        Scope::Index(i)
    }
}
