//! Emergent Presentation
//!
//! A package that specifies area markers and event handlers for the
//! Emergent User Interface library.

mod events;
pub use events::*;

mod presentation;
pub use presentation::*;

mod scope;
pub use scope::*;

/// A trait that describes types that can be surrounded by a scope identifiers.
pub trait Scoped {
    fn scoped(self, scope: Scope) -> Self;
}
