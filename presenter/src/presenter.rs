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
    BlendMode, Clip, Drawing, DrawingTarget, MeasureText, Paint, ReplaceWith, Shape, Transform,
};
use emergent_presentation::{Presentation, Scope};
use emergent_ui::FrameLayout;

/// The presenter is an ephemeral instance that is used to present one single frame.
pub struct Presenter<'a> {
    pub host: &'a mut Host,
    /// Boundaries of the presentation.
    boundary: FrameLayout,
    /// The current scope.
    scope: Vec<Scope>,
    /// The current presentation.
    presentation: Presentation,
}

impl Presenter<'_> {
    pub fn new(host: &mut Host, boundary: FrameLayout) -> Presenter {
        Presenter {
            host,
            boundary,
            scope: Vec::new(),
            presentation: Default::default(),
        }
    }

    pub fn scoped(&mut self, scope: Scope, f: impl FnOnce(&mut Presenter)) {
        self.scope.push(scope);
        f(self);
        let scope = self.scope.pop().unwrap();
        self.presentation.replace_with(|p| p.scoped(scope))
    }

    fn open_drawing(&mut self) -> &mut Drawing {
        self.presentation.open_drawing()
    }

    pub fn into_presentation(self) -> Presentation {
        self.presentation
    }
}

impl DrawingTarget for Presenter<'_> {
    fn fill(&mut self, paint: Paint, blend_mode: BlendMode) {
        self.open_drawing().fill(paint, blend_mode)
    }

    fn draw_shape(&mut self, shape: &Shape, paint: Paint) {
        self.open_drawing().draw_shape(shape, paint)
    }

    fn clip(&mut self, clip: &Clip, f: impl FnOnce(&mut Self)) {
        // TODO:
        unimplemented!()
    }

    fn transform(&mut self, transformation: &Transform, f: impl FnOnce(&mut Self)) {
        // TODO:
        unimplemented!()
    }
}
