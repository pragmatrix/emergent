//! Emergent Presentation
//!
//! A package that specifies area markers and event handlers for the
//! Emergent User Interface library.

mod events;
pub use events::*;

mod presentation;
pub use presentation::*;

/// A trait that describes types that can be surrounded by a scope identifiers.
// TODO: may use Cow?
pub trait Scoped {
    fn scoped(self, id: String) -> Self;
}
