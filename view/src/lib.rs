//! Emergent view model components.
//! A view model is defined to be the bridge between the application model and the visual
//! representation. It roles consists of:
//! - Presentation caching and rendering
//! - Layout
//! - Input Processing and Application Message generation.

mod change_set;
pub use change_set::*;

mod lazy_map;
pub use lazy_map::*;

mod indexed;
pub use indexed::*;

mod presentation;
pub use presentation::*;

mod view;
pub use view::*;
