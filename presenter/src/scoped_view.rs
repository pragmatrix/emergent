use crate::{View, ViewBuilder};
use emergent_drawing::{
    Bounds, DrawingBounds, DrawingFastBounds, MeasureText, Text, Transform, Transformed,
};
use emergent_presentation::PresentationScope;

/// This type allows a parent view building function to apply certain mutations to the
/// scoped view.
pub struct ScopedView<Msg> {
    view: View<Msg>,
}

impl<Msg> ScopedView<Msg> {
    pub(crate) fn new(view: View<Msg>) -> Self {
        Self { view }
    }

    fn map(self, f: impl Fn(View<Msg>) -> View<Msg>) -> Self {
        Self { view: f(self.view) }
    }

    pub fn fast_bounds(&self, measure: &dyn MeasureText) -> DrawingBounds {
        self.view.fast_bounds(measure)
    }

    pub fn transformed(self, transform: impl Into<Transform>) -> Self {
        let transform = transform.into();
        self.map(move |v| v.transformed(transform.clone()))
    }

    pub fn in_area(self) -> Self {
        self.map(|v| v.in_area())
    }

    pub fn presentation_scoped(self, scope: impl Into<PresentationScope>) -> Self {
        let scope = scope.into();
        self.map(|v| v.presentation_scoped(scope.clone()))
    }

    pub(crate) fn into_view(self) -> View<Msg> {
        self.view
    }
}
