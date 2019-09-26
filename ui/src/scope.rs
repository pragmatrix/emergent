use std::borrow::Cow;
use std::fmt;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum ScopeFragment {
    Named(Cow<'static, str>),
    Index(usize),
}

impl fmt::Display for ScopeFragment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ScopeFragment::*;
        match self {
            Named(n) => write!(f, "'{}'", n.to_owned()),
            Index(i) => write!(f, "{}", i),
        }
    }
}

impl From<&'static str> for ScopeFragment {
    fn from(str: &'static str) -> Self {
        ScopeFragment::Named(Cow::from(str))
    }
}

impl From<String> for ScopeFragment {
    fn from(str: String) -> Self {
        ScopeFragment::Named(Cow::from(str))
    }
}

impl From<usize> for ScopeFragment {
    fn from(i: usize) -> Self {
        ScopeFragment::Index(i)
    }
}
