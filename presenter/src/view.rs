use crate::{Context, GestureRecognizer};
use emergent_drawing::{
    Drawing, DrawingBounds, DrawingFastBounds, MeasureText, ReplaceWith, Transform, Transformed,
    Union,
};
use emergent_presentation::{Presentation, Scope, ScopePath, Scoped};
use std::cell::RefCell;

pub trait ViewRenderer<Msg> {
    fn render_view(&self, context: &mut Context<Msg>) -> View<Msg>;
}

pub struct View<Msg> {
    /// The presentation of the view.
    presentation: Presentation,

    /// Bounds representing the bounds of the presentation.
    ///
    /// When we construct a view solely based on Drawing or Presentation, the context / support needed
    /// computing bounds are not available, so we compute this lazily.
    bounds: RefCell<Option<DrawingBounds>>,

    /// The recognizers that are active.
    recognizers: Vec<(ScopePath, Box<dyn GestureRecognizer<Msg = Msg>>)>,
}

impl<Msg> View<Msg> {
    pub fn new() -> Self {
        Self {
            presentation: Default::default(),
            bounds: None.into(),
            recognizers: Default::default(),
        }
    }

    pub fn new_combined(views: impl IntoIterator<Item = View<Msg>>) -> View<Msg> {
        views.into_iter().fold(View::new(), |c, n| c.combined(n))
    }

    pub fn combined(mut self, right: View<Msg>) -> View<Msg> {
        self.presentation.push_on_top(right.presentation);
        self.recognizers.extend(right.recognizers);

        Self {
            presentation: self.presentation,
            // TODO: warnings: this is very problematic, causing bounds to be computed multiple times for
            // the same subtree of presentations.
            // Ideas:
            // - re-use combined bounds if each of the subview already has computed one.
            // - embed bounds in Presentations.
            bounds: None.into(),
            recognizers: self.recognizers,
        }
    }

    pub fn presentation(&self) -> &Presentation {
        &self.presentation
    }

    pub fn destructure(
        self,
    ) -> (
        Presentation,
        Vec<(ScopePath, Box<dyn GestureRecognizer<Msg = Msg>>)>,
    ) {
        (self.presentation, self.recognizers)
    }

    pub fn into_presentation(self) -> Presentation {
        self.destructure().0
    }
}

impl<Msg> Scoped for View<Msg> {
    fn scoped(mut self, scope: impl Into<Scope>) -> Self {
        let scope = scope.into();
        self.presentation.replace_with(|p| p.scoped(scope.clone()));
        self.recognizers
            .iter_mut()
            .for_each(|(p, _r)| p.replace_with(|p| p.scoped(scope.clone())));
        self
    }
}

impl<Msg> Transformed for View<Msg> {
    fn transformed(mut self, transform: impl Into<Transform>) -> Self {
        let transform = transform.into();
        self.bounds
            .get_mut()
            .map(|b| b.transformed(transform.clone()));
        self.presentation.replace_with(|p| p.transformed(transform));
        self
    }
}

impl<Msg> DrawingFastBounds for View<Msg> {
    fn fast_bounds(&self, measure: &dyn MeasureText) -> DrawingBounds {
        let mut r = DrawingBounds::Empty;
        self.bounds.replace_with(|b| match *b {
            Some(bounds) => {
                r = bounds;
                *b
            }
            None => {
                let bounds = self.presentation.fast_bounds(measure);
                r = bounds;
                Some(bounds)
            }
        });
        r
    }
}

impl<Msg> From<Presentation> for View<Msg> {
    fn from(presentation: Presentation) -> Self {
        Self {
            presentation,
            ..View::new()
        }
    }
}

impl<Msg> From<Drawing> for View<Msg> {
    fn from(drawing: Drawing) -> Self {
        Presentation::from(drawing).into()
    }
}
