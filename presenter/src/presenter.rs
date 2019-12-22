//! The presenter provides functionality to create presentations.
//!
//! These are:
//! - Scoping
//! - Event registration.
//! And planned are:
//! - Simple per-frame key / value caching
//! - culled, nested presentations.
//! - LOD sensitive recursive presentation.

use crate::{GestureRecognizer, Host};
use emergent_drawing::{
    Bounds, Drawing, DrawingFastBounds, MeasureText, Point, ReplaceWith, Text, Transform,
    Transformed, Vector,
};
use emergent_presentation::{Presentation, Scope};
use emergent_ui::FrameLayout;
use std::collections::HashMap;
use std::mem;

/// The presenter is an ephemeral instance that is used to present one single frame.
///
/// Implementation note: For simplicity of all the function signatures the clients will use,
/// I've decided to move Host inside the Presenter temporarily as long the frame is being built.
pub struct Presenter<Msg> {
    pub(crate) host: Host<Msg>,
    /// Boundaries of the presentation.
    boundary: FrameLayout,
    /// The current scope stack.
    scope: Vec<Scope>,
    /// The current presentation.
    pub(crate) presentation: Presentation,
    /// The current recognizers.
    /// TODO: this requires the complete scope to be copied.
    pub(crate) recognizers: HashMap<Vec<Scope>, Box<dyn GestureRecognizer<Msg = Msg>>>,
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

impl<Msg: 'static> Presenter<Msg> {
    pub fn new(host: Host<Msg>, boundary: FrameLayout) -> Self {
        Self {
            host,
            boundary,
            scope: Vec::new(),
            presentation: Default::default(),
            recognizers: Default::default(),
        }
    }

    /// Render a nested presentation into a scope and push it on top of the already existing presentation.
    /// A scope is meant to be a logical hierarhical structuring identity. Either a string, or an index.
    ///
    /// Its responsibilities are:
    /// - assisting the renderer to optimize by assuming that the content of a scope is related to a
    ///   scope with the same path from a previous run.
    /// - Defining the identity of areas.
    /// - Defining the identity of gesture and other local state.
    pub fn scoped(&mut self, scope: impl Into<Scope>, f: impl FnOnce(&mut Presenter<Msg>)) {
        self.scope.push(scope.into());
        let nested = self.nested(f);
        let scope = self.scope.pop().unwrap();
        self.presentation.push_on_top(nested.scoped(scope))
    }

    // Render a nested presentation, and define an area around it that is associated with the
    // current scope.
    pub fn area(&mut self, f: impl FnOnce(&mut Presenter<Msg>)) {
        let nested = self.nested(f);
        self.presentation.push_on_top(nested.in_area())
    }

    /// Render a gesture recognizer in the current scope.
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
    pub fn recognize(&mut self, recognizer: impl GestureRecognizer<Msg = Msg> + 'static) {
        match self.host.recognizers.remove(&self.scope) {
            Some(old_recognizer) => {
                debug!("replaced old recognizer at {:?}", self.scope);
                self.recognizers.insert(self.scope.clone(), old_recognizer);
            }
            None => {
                debug!("added new recognizer at {:?}", self.scope);
                self.recognizers
                    .insert(self.scope.clone(), Box::new(recognizer));
            }
        }
    }

    /// Render a nested presentation, transform it and push it on top of the already existing presentation.
    pub fn transformed(
        &mut self,
        transform: impl Into<Transform>,
        f: impl FnOnce(&mut Presenter<Msg>),
    ) {
        let nested = self.nested(f);
        self.presentation
            .push_on_top(nested.transformed(transform.into()))
    }

    /// Clear the current presentation, render a nested one, return it and restore the current presentation.
    fn nested(&mut self, f: impl FnOnce(&mut Presenter<Msg>)) -> Presentation {
        let presentation = mem::replace(&mut self.presentation, Presentation::Empty);
        f(self);
        mem::replace(&mut self.presentation, presentation)
    }

    fn on_top(&mut self, f: impl FnOnce(&mut Presenter<Msg>)) {
        let nested = self.nested(f);
        self.presentation.push_on_top(nested)
    }

    pub fn draw(&mut self, drawing: Drawing) {
        self.open_drawing().replace_with(|d| d.below(drawing))
    }

    fn open_drawing(&mut self) -> &mut Drawing {
        self.presentation.open_drawing()
    }

    pub fn stack_items<Item>(
        &mut self,
        direction: Direction,
        items: &[Item],
        f: impl Fn(&mut Presenter<Msg>, (usize, &Item)),
    ) {
        self.stack(direction, items.len(), |presenter, i| {
            f(presenter, (i, &items[i]))
        })
    }

    pub fn stack_f(&mut self, direction: Direction, fs: &[&dyn Fn(&mut Presenter<Msg>)]) {
        self.stack(direction, fs.len(), |presenter, i| (fs[i])(presenter))
    }

    /// Stack a number of presentations in the `direction` given by a Vector.
    pub fn stack(
        &mut self,
        direction: Direction,
        count: usize,
        f: impl Fn(&mut Presenter<Msg>, usize),
    ) {
        let direction = direction.to_vector();
        let mut p = Point::default();
        for i in 0..count {
            self.scoped(i, |presenter| {
                let nested = presenter.nested(|presenter| f(presenter, i));
                let drawing_bounds = nested.fast_bounds(presenter);
                if let Some(bounds) = drawing_bounds.as_bounds() {
                    let align = -bounds.point.to_vector();
                    let nested = nested.transformed((p + align).to_vector());
                    presenter.presentation.push_on_top(nested);
                    p += Vector::from(bounds.extent) * direction;
                }
            })
        }
    }

    pub fn into_presentation(self) -> Presentation {
        self.presentation
    }

    /// Takes the current presentation out of the presenter and replaces the current one with an
    /// empty presentation.
    pub fn take_presentation(&mut self) -> Presentation {
        mem::replace(&mut self.presentation, Presentation::Empty)
    }
}

// TODO: this is a good candidate for a per frame cache.
impl<Msg> MeasureText for Presenter<Msg> {
    fn measure_text(&self, text: &Text) -> Bounds {
        self.host.support.measure_text(text)
    }
}
