use crate::{Context, ContextPath, ContextScope, RecognizerRecord, ScopedState};
use emergent_drawing::{
    Drawing, DrawingBounds, DrawingFastBounds, MeasureText, ReplaceWith, Transform, Transformed,
};
use emergent_presentation::{Presentation, PresentationScope, Scoped};
use std::any::Any;
use std::cell::RefCell;

pub mod scroll;

pub trait ViewRenderer<Msg> {
    fn render_view(&self, context: Context) -> View<Msg>;
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
    recognizers: Vec<RecognizerRecord<Msg>>,

    /// The captured states of all the context scopes.
    /// TODO: may put them into ScopedStates?
    states: Vec<(ContextPath, Box<dyn Any>)>,
}

impl<Msg> Default for View<Msg> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Msg> View<Msg> {
    pub fn new() -> Self {
        Self {
            presentation: Default::default(),
            bounds: None.into(),
            recognizers: Default::default(),
            states: Default::default(),
        }
    }

    pub fn new_combined(views: impl IntoIterator<Item = View<Msg>>) -> View<Msg> {
        views.into_iter().fold(View::new(), |c, n| c.combined(n))
    }

    pub fn combined(mut self, right: View<Msg>) -> View<Msg> {
        self.presentation.push_on_top(right.presentation);
        self.recognizers.extend(right.recognizers);
        self.states.extend(right.states);

        Self {
            presentation: self.presentation,
            // TODO: warnings: this is very problematic, causing bounds to be computed multiple times for
            // the same subtree of presentations.
            // Ideas:
            // - re-use combined bounds if each of the subview already has computed one.
            // - embed bounds in Presentations.
            bounds: None.into(),
            recognizers: self.recognizers,
            states: self.states,
        }
    }

    /// Put the presentation inside an area.
    pub fn in_area(self) -> Self {
        Self {
            presentation: self.presentation.in_area(),
            ..self
        }
    }

    pub(crate) fn record_recognizer(mut self, recognizer: RecognizerRecord<Msg>) -> Self {
        self.recognizers.push(recognizer);
        self
    }

    pub fn presentation(&self) -> &Presentation {
        &self.presentation
    }

    pub(crate) fn destructure(
        self,
    ) -> (Presentation, Vec<RecognizerRecord<Msg>>, Vec<ScopedState>) {
        (self.presentation, self.recognizers, self.states)
    }

    pub fn into_presentation(self) -> Presentation {
        self.destructure().0
    }

    pub(crate) fn presentation_scoped(mut self, scope: impl Into<PresentationScope>) -> Self {
        let scope = scope.into();
        self.presentation.replace_with(|p| p.scoped(scope.clone()));
        self.recognizers
            .iter_mut()
            .for_each(|r| r.replace_with(|r| r.presentation_scoped(scope.clone())));
        self
    }

    pub fn context_scoped(mut self, scope: impl Into<ContextScope>) -> Self {
        let scope = scope.into();
        // This effectively promotes the view to a context above. Assuming that we don't need
        // to reuse nested store data, we can clear it.
        self.states
            .iter_mut()
            .for_each(|(s, _)| s.replace_with(|s| s.scoped(scope.clone())));
        self.recognizers
            .iter_mut()
            .for_each(|r| r.replace_with(|r| r.context_scoped(scope.clone())));

        self
    }

    pub fn store_states(mut self, states: impl IntoIterator<Item = Box<dyn Any>>) -> Self {
        self.states
            .extend(states.into_iter().map(|s| (ContextPath::new(), s)));
        self
    }

    pub fn store_state(mut self, state: impl Any + 'static) -> Self {
        self.states.push((ContextPath::new(), Box::new(state)));
        self
    }
}

impl<Msg> Transformed for View<Msg> {
    fn transformed(mut self, transform: impl Into<Transform>) -> Self {
        let transform = transform.into();
        self.bounds
            .replace_with(|b| b.map(|b| b.transformed(transform.clone())));
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
