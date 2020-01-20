//! The presenter provides functionality to create presentations.
//!
//! These are:
//! - Scoping
//! - Event registration.
//! And planned are:
//! - Simple per-frame key / value caching
//! - culled, nested presentations.
//! - LOD sensitive recursive presentation.

use crate::{ScopeState, Support, View};
use emergent_drawing::{Bounds, MeasureText, Text, Vector};
use emergent_presentation::Scope;
use emergent_ui::FrameLayout;
use std::rc::Rc;

/// The context is an ephemeral instance that is used to present something inside a space that
/// is defined by a named or indexed scope.
pub struct Context {
    support: Rc<Support>,
    /// Boundaries of the presentation.
    boundary: FrameLayout,
    /// The state tree from the previous view rendering process.
    previous: ScopeState,
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
    pub fn new(support: Rc<Support>, boundary: FrameLayout, previous: ScopeState) -> Self {
        Self {
            support,
            boundary,
            previous,
        }
    }

    /// Render a nested view into the given scope.
    /// A scope is meant to be a logical hierarchical structuring identity. Either a string, or an index.
    ///
    /// The return value _is_ the nested view, for the re-rendering to work, the nested view has to be added to
    /// a parent view with the same scope.
    ///
    /// TODO: we can probably just move the context here into the function `f` or even just return a nested context for
    ///       consumption.
    pub fn nested<Msg>(
        &mut self,
        scope: impl Into<Scope>,
        view: impl FnOnce(&mut Context) -> View<Msg>,
    ) -> View<Msg> {
        let scope = scope.into();
        let previous = self
            .previous
            .nested
            .remove(&scope)
            .unwrap_or_else(ScopeState::new);

        let mut nested_context = Context::new(self.support.clone(), self.boundary, previous);
        view(&mut nested_context)
    }

    /*
    /// Stick or reuse a typed component in the current scope.
    pub fn resolve<C: 'static>(&mut self, construct: impl FnOnce() -> C) -> &mut C {
        let type_id = any::TypeId::of::<C>();
        // TODO: prevent this clone!
        let v = match self.previous.components.remove(&type_id) {
            Some(reusable) => reusable,
            // TODO: why downcast later when we directly create the concrete instance here.
            None => Box::new(construct()),
        };

        // TODO: find a one-step process for inserting and getting a mutable reference to value
        // (using entry)?.
        self.current.components.insert(type_id, v);
        self.current
            .components
            .get_mut(&type_id)
            .unwrap()
            .deref_mut()
            .downcast_mut::<C>()
            .unwrap()
    }

    */
}

// TODO: this is a good candidate for a per frame cache.
impl MeasureText for Context {
    fn measure_text(&self, text: &Text) -> Bounds {
        self.support.measure_text(text)
    }
}
