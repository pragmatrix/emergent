//! The presenter provides functionality to create presentations.
//!
//! These are:
//! - Scoping
//! - Event registration.
//! And planned are:
//! - Simple per-frame key / value caching
//! - culled, nested presentations.
//! - LOD sensitive recursive presentation.

use crate::Host;
use emergent_drawing::{
    Bounds, Drawing, DrawingFastBounds, MeasureText, Point, ReplaceWith, Text, Transform,
    Transformed, Vector,
};
use emergent_presentation::{Presentation, Scope};
use emergent_ui::FrameLayout;
use std::mem;

/// The presenter is an ephemeral instance that is used to present one single frame.
///
/// Implementation note: For simplicity of all the function signatures the clients will use,
/// I've decided to move Host inside the Presenter temporarily as long the frame is being built.
pub struct Presenter {
    host: Host,
    /// Boundaries of the presentation.
    boundary: FrameLayout,
    /// The current scope stack.
    scope: Vec<Scope>,
    /// The current presentation.
    presentation: Presentation,
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

impl Presenter {
    pub fn new(host: Host, boundary: FrameLayout) -> Presenter {
        Presenter {
            host,
            boundary,
            scope: Vec::new(),
            presentation: Default::default(),
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
    pub fn scoped(&mut self, scope: impl Into<Scope>, f: impl FnOnce(&mut Presenter)) {
        self.scope.push(scope.into());
        let nested = self.nested(f);
        let scope = self.scope.pop().unwrap();
        self.presentation.push_on_top(nested.scoped(scope))
    }

    // Render a nested presentation, and define an area around it that is associated with the
    // current scope.
    pub fn area(&mut self, f: impl FnOnce(&mut Presenter)) {
        let nested = self.nested(f);
        self.presentation.push_on_top(nested.in_area())
    }

    /// Render a nested presentation, transform it and push it on top of the already existing presentation.
    pub fn transformed(&mut self, transform: impl Into<Transform>, f: impl FnOnce(&mut Presenter)) {
        let nested = self.nested(f);
        self.presentation
            .push_on_top(nested.transformed(transform.into()))
    }

    /// Clear the current presentation, render a nested one, return it and restore the current presentation.
    fn nested(&mut self, f: impl FnOnce(&mut Presenter)) -> Presentation {
        let presentation = mem::replace(&mut self.presentation, Presentation::Empty);
        f(self);
        mem::replace(&mut self.presentation, presentation)
    }

    fn on_top(&mut self, f: impl FnOnce(&mut Presenter)) {
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
        f: impl Fn(&mut Presenter, (usize, &Item)),
    ) {
        self.stack(direction, items.len(), |presenter, i| {
            f(presenter, (i, &items[i]))
        })
    }

    pub fn stack_f(&mut self, direction: Direction, fs: &[&dyn Fn(&mut Presenter)]) {
        self.stack(direction, fs.len(), |presenter, i| (fs[i])(presenter))
    }

    /// Stack a number of presentations in the `direction` given by a Vector.
    pub fn stack(&mut self, direction: Direction, count: usize, f: impl Fn(&mut Presenter, usize)) {
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

    /// Converts the Presenter back into the host and the resulting presentation.
    pub fn into_host_and_presentation(self) -> (Host, Presentation) {
        (self.host, self.presentation)
    }

    pub fn into_presentation(self) -> Presentation {
        self.into_host_and_presentation().1
    }

    /// Takes the current presentation out of the presenter and replaces the current one with an
    /// empty presentation.
    pub fn take_presentation(&mut self) -> Presentation {
        mem::replace(&mut self.presentation, Presentation::Empty)
    }
}

// TODO: this is a good candidate for a per frame cache.
impl MeasureText for Presenter {
    fn measure_text(&self, text: &Text) -> Bounds {
        self.host.support.measure_text(text)
    }
}
