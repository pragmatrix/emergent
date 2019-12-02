//! The presenter provides functionality to create presentations.
//!
//! These are:
//! - Scoping
//! - Event registration.
//! And planned are:
//! - Simple per-frame key / value caching
//! - culled, nested presentations.
//! - LOD sensitive recursive presentation.

use crate::{Host, Support};
use emergent_drawing::{
    BlendMode, Bounds, Clip, Drawing, DrawingTarget, MeasureText, Paint, ReplaceWith, Shape, Text,
    Transform,
};
use emergent_presentation::{Presentation, Scope};
use emergent_ui::FrameLayout;

/// The presenter is an ephemeral instance that is used to present one single frame.
///
/// Implementation note: For simplicity of all the function signatures the clients will use,
/// I've decided to move Host inside the Presenter temporarily as long the frame is being built.
pub struct Presenter {
    host: Host,
    /// Boundaries of the presentation.
    boundary: FrameLayout,
    /// The current scope.
    scope: Vec<Scope>,
    /// The current presentation.
    presentation: Presentation,
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

    pub fn scoped(&mut self, scope: Scope, f: impl FnOnce(&mut Presenter)) {
        self.scope.push(scope);
        f(self);
        let scope = self.scope.pop().unwrap();
        self.presentation.replace_with(|p| p.scoped(scope))
    }

    fn open_drawing(&mut self) -> &mut Drawing {
        self.presentation.open_drawing()
    }

    // Present a presentation on top of everything that was presented before.
    // TODO: this won't be possible in future versions, because Presentation will
    // be an internal abstraction. In the future, Presentation's are built solely by
    // calling tracing function in the Presenter.
    pub fn present(&mut self, presentation: Presentation) {
        self.presentation.push_on_top(presentation)
    }

    /// Converts the Presenter back into the host and the resulting presentation.
    pub fn into_host_and_presentation(self) -> (Host, Presentation) {
        (self.host, self.presentation)
    }

    pub fn into_presentation(self) -> Presentation {
        self.into_host_and_presentation().1
    }
}

// TODO: this is a good candidate for a per frame cache.
impl MeasureText for Presenter {
    fn measure_text(&self, text: &Text) -> Bounds {
        self.host.support.measure_text(text)
    }
}

impl DrawingTarget for Presenter {
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
