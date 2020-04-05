use std::borrow::Cow;
use std::fmt;
use std::marker::PhantomData;

/// A scope path contains a sequence of scopes that together describes a branch path  
/// inside a nested data structure.
///
/// The type argument `T` is used to discriminate different types of scope paths.
///
/// # Performance
///
/// - Even though `VecDeque` would probably be a better type, we have uses in which we prefer to
///   access parts of paths through slices.
/// - Path lengths should not exceed a few elements, so inserting at the beginning might not
///   be that expensive.
pub type ScopePath<T> = Vec<Scope<T>>;

/// A trait that can be implemented by types that can be extended with / put into a scope.
pub trait Scoped<T> {
    fn scoped(self, scope: impl Into<Scope<T>>) -> Self;
}

/// A scope is defined to be either by a `Name` or an `Index`.
///
/// The type argument `T` is used to discriminate different types of scopes.
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

impl<M> Scoped<M> for Vec<Scope<M>> {
    fn scoped(mut self, scope: impl Into<Scope<M>>) -> Self {
        self.insert(0, scope.into());
        self
    }
}
