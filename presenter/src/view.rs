use crate::{ContextPath, ContextScope, ProcessorRecord, ScopedState, ViewBuilder};
use emergent_drawing::{
    Bounds, DrawingBounds, DrawingFastBounds, MeasureText, Rect, ReplaceWith, Transform,
    Transformed,
};
use emergent_presentation::{Presentation, PresentationScope, Scoped};
use std::any::Any;
use std::cell::RefCell;

pub mod scroll;
pub mod tab;

pub trait ViewRenderer<Msg> {
    fn render_view(&self, builder: ViewBuilder<Msg>) -> View<Msg>;
}

pub struct View<Msg> {
    /// The presentation of the view.
    presentation: Presentation,

    /// Bounds representing the bounds of the presentation.
    ///
    /// When we construct a view solely based on Drawing or Presentation, the context / support needed
    /// computing bounds are not available, so we compute this lazily.
    bounds: RefCell<Option<DrawingBounds>>,

    /// The input processors that are active.
    processors: Vec<ProcessorRecord<Msg>>,

    /// The captured states of all the context scopes.
    /// TODO: may put them into ScopedStates?
    states: Vec<(ContextPath, Box<dyn Any>)>,
}

impl<Msg> View<Msg> {
    pub(crate) fn new(
        presentation: Presentation,
        processors: Vec<ProcessorRecord<Msg>>,
        states: Vec<(ContextPath, Box<dyn Any>)>,
    ) -> Self {
        Self {
            presentation,
            bounds: None.into(),
            processors,
            states,
        }
    }

    pub(crate) fn new_combined(
        container: View<Msg>,
        nested: impl IntoIterator<Item = View<Msg>>,
    ) -> View<Msg> {
        nested.into_iter().fold(container, |c, n| c.combined(n))
    }

    pub(crate) fn combined(mut self, right: View<Msg>) -> View<Msg> {
        self.presentation.push_on_top(right.presentation);
        // TODO: this is very problematic, causing bounds to be computed multiple times for
        // the same subtree of presentations.
        // Ideas:
        // - re-use combined bounds if each of the subview already has computed one.
        // - embed bounds in Presentations.
        self.bounds = None.into();
        self.processors.extend(right.processors);
        self.states.extend(right.states);

        self
    }

    /// Put the presentation inside an area.
    pub fn in_area(self) -> Self {
        Self {
            presentation: self.presentation.in_area(),
            ..self
        }
    }

    pub fn presentation(&self) -> &Presentation {
        &self.presentation
    }

    pub(crate) fn presentation_mut(&mut self) -> &mut Presentation {
        &mut self.presentation
    }

    pub(crate) fn destructure(self) -> (Presentation, Vec<ProcessorRecord<Msg>>, Vec<ScopedState>) {
        (self.presentation, self.processors, self.states)
    }

    pub fn into_presentation(self) -> Presentation {
        self.destructure().0
    }

    pub(crate) fn presentation_scoped(mut self, scope: impl Into<PresentationScope>) -> Self {
        let scope = scope.into();
        self.presentation.replace_with(|p| p.scoped(scope.clone()));
        self.processors
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
        self.processors
            .iter_mut()
            .for_each(|r| r.replace_with(|r| r.context_scoped(scope.clone())));

        self
    }

    pub fn with_states(mut self, states: impl IntoIterator<Item = Box<dyn Any>>) -> Self {
        self.states
            .extend(states.into_iter().map(|s| (ContextPath::new(), s)));
        self
    }

    pub fn with_state(mut self, state: impl Any + 'static) -> Self {
        self.states.push((ContextPath::new(), Box::new(state)));
        self
    }

    /// Returns this view trimmed to a boundary.
    ///
    /// This removes
    /// - parts of the presentation that are completely outside the boundary rectangle.
    /// - processors of which their presentation scope does not exist anymore and don't
    ///   have any active subscriptions.
    pub fn trimmed(self, bounds: Bounds, measure: &dyn MeasureText) -> Self {
        let (presentation, _) = self.presentation.trimmed(bounds, measure);
        Self {
            presentation,
            ..self
        }
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
