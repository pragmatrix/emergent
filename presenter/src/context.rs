//! The `Context` type provides functionality to create views.
//!
//! These are:
//! - Scoping nested views.
//! - Input processor registration.
//! - Function local view state.
//! And planned are:
//! - Simple per-frame key / value caching
//! - culled, nested presentations.
//! - LOD sensitive recursive presentation.

use crate::{ScopedStore, Support, View};
use emergent_drawing::{Bounds, MeasureText, Point, Rect, Text, Vector};
use emergent_presentation::{Scope, ScopePath};
use emergent_ui::FrameLayout;
use std::any;
use std::any::TypeId;
use std::rc::Rc;

// Can't use `Context` here for marking scopes, because it does not support certain trait which Scope / ScopePath needs
// to.
pub type ContextScope = Scope<ContextMarker>;
pub type ContextPath = ScopePath<ContextMarker>;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, Default)]
pub struct ContextMarker;

/// An ephemeral type that is used to present views inside a space that
/// is defined by a named or indexed scope.
///
/// TODO: may rename to ViewState or (View)Builder?
#[derive(Debug)]
pub struct Context {
    support: Rc<Support>,
    /// Physical boundaries of the presentation.
    /// TODO: do we need that here?
    boundary: FrameLayout,
    /// Logical boundaries of the presentation.
    view_bounds: Rect,
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
        let (width, height) = boundary.dimensions;
        Self {
            support,
            boundary,
            view_bounds: Rect::from((
                Point::from((0, 0)),
                Vector::from((width as isize, height as isize)),
            )),
            previous,
        }
    }

    pub fn scoped<Msg>(
        &mut self,
        scope: impl Into<ContextScope>,
        create_content: impl FnOnce(Context) -> View<Msg>,
    ) -> View<Msg> {
        let (_r, view) = self.scoped_r(scope, |c| ((), create_content(c)));
        view
    }
    /// Produce a view inside the scoped context.
    ///
    /// A `ContextScope` is meant to be resemble the function call hierarchy and is not necessarily related to the
    /// resulting view graph.
    ///
    /// The return value _is_ the view that was produced inside the scoped context.
    pub fn scoped_r<Msg, R>(
        &mut self,
        scope: impl Into<ContextScope>,
        create_content: impl FnOnce(Context) -> (R, View<Msg>),
    ) -> (R, View<Msg>) {
        let scope = scope.into();
        let previous = self
            .previous
            .remove_scope(scope.clone())
            .unwrap_or_else(ScopedStore::new);

        let nested_context = Context::new(self.support.clone(), self.boundary, previous);
        let (r, view) = create_content(nested_context);
        let view = view.context_scoped(scope);
        (r, view)
    }

    pub fn with_state<S: 'static, Msg>(
        &mut self,
        construct: impl FnOnce() -> S,
        with_state: impl FnOnce(Context, &S) -> View<Msg>,
    ) -> View<Msg> {
        self.with_state_r(construct, |c, s| ((), with_state(c, s)))
            .1
    }

    /// Calls a function that maintains uses view state and generates a view.
    ///
    /// If there is no state available at the current context scope, `construct` is called to generate a new one.
    /// If there is a state available, the previous state is recycled and passed to the `with_state` function.
    pub fn with_state_r<S: 'static, Msg, R>(
        &mut self,
        construct: impl FnOnce() -> S,
        with_state: impl FnOnce(Context, &S) -> (R, View<Msg>),
    ) -> (R, View<Msg>) {
        let state = self.recycle_state().unwrap_or_else(construct);
        let scope: ContextScope = any::type_name::<S>().into();
        let (r, view) = self.scoped_r(scope, |ctx| with_state(ctx, &state));
        (r, view.with_state(state))
    }

    /// Tries to recycle a typed state from the current context. If successful, the typed state is removed.
    pub(crate) fn recycle_state<S: 'static>(&mut self) -> Option<S> {
        match self.previous.remove_state() {
            None => {
                warn!(
                    "failed to recycle state: {:?} {:?}, states available: {:?}",
                    TypeId::of::<S>(),
                    any::type_name::<S>(),
                    self.previous
                );
                None
            }
            Some(r) => Some(r),
        }
    }

    pub fn support(&self) -> Rc<Support> {
        self.support.clone()
    }

    pub fn view_bounds(&self) -> Rect {
        self.view_bounds.clone()
    }
}

// TODO: this is a good candidate for a per frame cache.
impl MeasureText for Context {
    fn measure_text(&self, text: &Text) -> Bounds {
        self.support.measure_text(text)
    }
}
