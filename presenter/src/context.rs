//! The presenter provides functionality to create presentations.
//!
//! These are:
//! - Scoping
//! - Event registration.
//! And planned are:
//! - Simple per-frame key / value caching
//! - culled, nested presentations.
//! - LOD sensitive recursive presentation.

use crate::{ScopedState, Support, View};
use emergent_drawing::{Bounds, MeasureText, Text, Vector};
use emergent_presentation::{Scope, ScopePath};
use emergent_ui::FrameLayout;
use std::rc::Rc;

// Can't use Context here, because it does not support certain trait which Scope / ScopePath needs to.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct ContextMarker;

pub type ContextScope = Scope<ContextMarker>;
pub type ContextPath = ScopePath<ContextMarker>;

/// The context is an ephemeral instance that is used to present something inside a space that
/// is defined by a named or indexed scope.
pub struct Context {
    support: Rc<Support>,
    /// Boundaries of the presentation.
    boundary: FrameLayout,
    /// The state tree from the previous view rendering process.
    previous: ScopedState,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Direction {
    Row,
    RowReverse,
    Column,
    ColumnReverse,
}

impl Direction {
    pub fn to_vector(self) -> Vector {
        match self {
            Direction::Row => Vector::new(1.0, 0.0),
            Direction::RowReverse => Vector::new(-1.0, 0.0),
            Direction::Column => Vector::new(0.0, 1.0),
            Direction::ColumnReverse => Vector::new(0.0, -1.0),
        }
    }
}

impl Context {
    pub fn new(support: Rc<Support>, boundary: FrameLayout, previous: ScopedState) -> Self {
        Self {
            support,
            boundary,
            previous,
        }
    }

    /// Produce a view inside the given scoped context.
    ///
    /// A scope is meant to be a hierarchical structuring identity that resembles the function call hierarchy and is not
    /// necessarily related to the resulting view graph.
    ///
    /// A scope is either a string or an index.
    ///
    /// The return value _is_ the view that was produced inside the scoped context.
    ///
    /// TODO: we can probably just move the context here into the function `f` or even just return a nested context for
    ///       consumption.
    pub fn scoped<Msg>(
        &mut self,
        scope: impl Into<ContextScope>,
        view: impl FnOnce(Context) -> View<Msg>,
    ) -> View<Msg> {
        let scope = scope.into();
        let previous = self
            .previous
            .remove_scope(scope.clone())
            .unwrap_or_else(ScopedState::new);

        let nested_context = Context::new(self.support.clone(), self.boundary, previous);
        view(nested_context).context_scoped(scope)
    }

    /// Tries to reuse a typed state from the current scoped context. This removes the typed state.
    pub fn reuse<S: 'static>(&mut self, construct: impl FnOnce() -> S) -> S {
        self.previous.remove_state().unwrap_or_else(construct)
    }

    /// Calls a function that uses a state.
    ///
    /// This marks the state as live and stores it alongside the returned view if the context is resolved.
    /// TODO: the state should probably not be able to be modified, so it can be passed as a reference only
    ///       to `with_state`?
    /// TODO: if the return type is a View<> could we embed the state into the view returned and don't need to
    ///       store it in the Context?
    pub fn with_state<S: 'static, Msg>(
        mut self,
        construct: impl FnOnce() -> S,
        with_state: impl FnOnce(Context, S) -> (S, View<Msg>),
    ) -> View<Msg> {
        // TODO: can we actually test if the state has been modified?
        let state = self.reuse(construct);
        let (new_state, view) = with_state(self, state);
        view.store_state(new_state)
    }

    pub fn support(&self) -> Rc<Support> {
        self.support.clone()
    }
}

// TODO: this is a good candidate for a per frame cache.
impl MeasureText for Context {
    fn measure_text(&self, text: &Text) -> Bounds {
        self.support.measure_text(text)
    }
}
