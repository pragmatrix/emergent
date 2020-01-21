use crate::Scoped;
use std::borrow::Cow;
use std::fmt;
use std::marker::PhantomData;

pub type ScopePath<T> = Vec<Scope<T>>;

impl<T> Scoped<T> for Vec<Scope<T>> {
    fn scoped(mut self, scope: impl Into<Scope<T>>) -> Self {
        // TODO: rather unfortunate that we need to prepend here. May use another datatype.
        self.insert(0, scope.into());
        self
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum Scope<T> {
    Name(Cow<'static, str>, PhantomData<*const T>),
    Index(usize),
}

impl<T> fmt::Display for Scope<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Scope::*;
        match self {
            Name(n, _) => write!(f, "'{}'", n.to_owned()),
            Index(i) => write!(f, "{}", i),
        }
    }
}

impl<T> From<&'static str> for Scope<T> {
    fn from(str: &'static str) -> Self {
        Scope::Name(Cow::from(str), PhantomData)
    }
}

impl<T> From<String> for Scope<T> {
    fn from(str: String) -> Self {
        Scope::Name(Cow::from(str), PhantomData)
    }
}

impl<T> From<&String> for Scope<T> {
    fn from(str: &String) -> Self {
        str.clone().into()
    }
}

impl<T> From<usize> for Scope<T> {
    fn from(i: usize) -> Self {
        Scope::Index(i)
    }
}
