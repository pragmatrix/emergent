//! Emergent Presentation
//!
//! A package that specifies area markers and event handlers for the
//! Emergent User Interface library.

mod presentation;
pub use presentation::*;

mod scope;
pub use scope::*;

/// A trait that describes types that can be surrounded by a scope identifiers.
pub trait Scoped<T> {
    fn scoped(self, scope: impl Into<Scope<T>>) -> Self;
}
