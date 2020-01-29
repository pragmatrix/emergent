//! The builder provides functionality to create views.
//!
//! These are:
//! - Scoping nested views.
//! - Recognizer registration.
//! - Function local view state (this is cool).
//! And planned are:
//! - Simple per-frame key / value caching
//! - culled, nested presentations.
//! - LOD sensitive recursive presentation.

use crate::{GestureRecognizer, RecognizerRecord, ScopedStore, Support, View};
use emergent_drawing::{Bounds, MeasureText, Text, Vector};
use emergent_presentation::{Scope, ScopePath};
use emergent_ui::FrameLayout;
use std::any;
use std::any::Any;
use std::rc::Rc;

// Can't use Context here, because it does not support certain trait which Scope / ScopePath needs to.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct ContextMarker;

pub type ContextScope = Scope<ContextMarker>;
pub type ContextPath = ScopePath<ContextMarker>;

/// The context is an ephemeral instance that is used to present something inside a space that
/// is defined by a named or indexed scope.
///
/// TODO: may rename to ViewState or (View)Builder?
pub struct Context {
    support: Rc<Support>,
    /// Boundaries of the presentation.
    boundary: FrameLayout,
    /// The state tree from the previous view rendering process.
    previous: ScopedStore,
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
    pub fn new(support: Rc<Support>, boundary: FrameLayout, previous: ScopedStore) -> Self {
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
            .unwrap_or_else(ScopedStore::new);

        let nested_context = Context::new(self.support.clone(), self.boundary, previous);
        view(nested_context).context_scoped(scope)
    }

    /// Calls a function that uses a state and generates a view.
    pub fn with_state<S: 'static, Msg>(
        &mut self,
        construct: impl FnOnce() -> S,
        with_state: impl FnOnce(Context, &S) -> View<Msg>,
    ) -> View<Msg> {
        let state = self.recycle_state().unwrap_or_else(construct);
        let scope: ContextScope = any::type_name::<S>().into();
        let view = self.scoped(scope, |ctx| with_state(ctx, &state));
        view.store_state(state)
    }

    /// Adds a recognizer to the view.
    ///
    /// May reuse a recognizer with the same type from the context.
    pub fn add_recognizer<Msg, R>(
        &mut self,
        view: View<Msg>,
        construct: impl FnOnce() -> R,
    ) -> View<Msg>
    where
        Msg: 'static,
        R: GestureRecognizer<Event = Msg> + 'static,
    {
        // recycling tries to pull out  a boxed version of the recognizer, because we can not cast directly
        // from the `GestureRecognizer` trait to the `Any` trait when we push down states.
        let r = self.recycle_state::<R>();
        let r = r.unwrap_or_else(construct);

        let record = RecognizerRecord::new(
            Box::new(r),
            Box::new(|b: &mut Box<dyn Any>| b.downcast_mut::<R>().unwrap()),
        );
        view.record_recognizer(record)
    }

    /// Tries to recycle a typed state from the current context. This removes the typed state.
    fn recycle_state<S: 'static>(&mut self) -> Option<S> {
        match self.previous.remove_state() {
            None => {
                trace!("failed to recycle state: {:?}", any::type_name::<S>());
                None
            }
            Some(r) => Some(r),
        }
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
