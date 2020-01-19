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
pub struct Context<Msg> {
    support: Rc<Support>,
    /// Boundaries of the presentation.
    boundary: FrameLayout,
    /// The state tree from the previous view rendering process.
    previous: ScopeState<Msg>,
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

impl<Msg> Context<Msg> {
    pub fn new(support: Rc<Support>, boundary: FrameLayout, previous: ScopeState<Msg>) -> Self {
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
    pub fn nested(
        &mut self,
        scope: impl Into<Scope>,
        view: impl FnOnce(&mut Context<Msg>) -> View<Msg>,
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

    // Render a nested presentation, and define an area around it that is associated with the
    // current scope.
    /*
    pub fn area(&mut self, f: impl FnOnce(&mut Presenter<Msg>)) {
        let nested = self.nested(f);
        self.presentation.push_on_top(nested.in_area())
    }
    */

    /*

    /// Present a gesture recognizer in the current scope.
    ///
    /// If there is no area in the current scope, the whole scope is considered the area of the gesture
    /// recognizer.
    ///
    /// If there multiple areas in the current scope. All the areas decide which events are delivered
    /// to the gesture recognizer.
    ///
    /// Re-rendering the same type of gesture recognizer in the same scope does not update or reset the
    /// state of the gesture recognizer (for now).
    ///
    /// If a gesture recognizer disappears from a scope, it will be removed from the presentation.

        pub fn recognize(&mut self, recognizer: impl GestureRecognizer<Msg = Msg> + 'static)
        where
            Msg: 'static,
        {
            let key = recognizer.type_id();
            match self.previous.recognizers.remove(&key) {
                Some(previous_recognizer) => {
                    debug!("reused recognizer");
                    self.current.recognizers.insert(key, previous_recognizer);
                }
                None => {
                    debug!("added new recognizer");
                    self.current.recognizers.insert(key, Box::new(recognizer));
                }
            }
        }
    */

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
impl<Msg> MeasureText for Context<Msg> {
    fn measure_text(&self, text: &Text) -> Bounds {
        self.support.measure_text(text)
    }
}
