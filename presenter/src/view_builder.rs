use crate::input_processor::Subscriber;
use crate::{
    Context, ContextPath, ContextScope, InputProcessor, ProcessorRecord, ScopedView, Support, View,
};
use emergent_drawing::{
    Bounds, DrawingBounds, DrawingFastBounds, MeasureText, Rect, Text, Transform, Transformed,
};
use emergent_presentation::{Presentation, PresentationScope};
use emergent_ui::WindowMessage;
use std::any::Any;
use std::iter;
use std::ops::Deref;
use std::rc::Rc;

pub struct ViewBuilder<Msg> {
    context: Context,

    /// The recycled or newly generated processors that are active.
    processors: Vec<ProcessorRecord<Msg>>,

    /// The recycled or newly captured states of all the context scopes.
    states: Vec<Box<dyn Any>>,
}

impl<Msg> ViewBuilder<Msg> {
    // TODO: make this pub(crate) (used in from_test_environment())
    pub fn new(context: Context) -> Self {
        Self {
            context,
            processors: Vec::new(),
            states: Vec::new(),
        }
    }

    pub fn support(&self) -> &Rc<Support> {
        self.context.support()
    }

    pub fn view_bounds(&self) -> Rect {
        self.context.view_bounds()
    }

    /// Recycles a typed state from the current context or constructs it and binds it to the current view builder,
    /// when it was not available before.
    pub fn use_state<S: 'static>(&mut self, construct: impl FnOnce() -> S) -> &S {
        let state = self.context.recycle_state().unwrap_or_else(construct);
        // TODO: can the following two be combined?
        self.set_state(state)
    }

    /// Sets a new state and makes it available to the current Context.
    /// Contrary to `use_state`, this state block is never recycled.
    ///
    /// Setting a state can be useful to provide additional information to input processors.
    pub fn set_state<S: 'static>(&mut self, state: S) -> &S {
        let b = Box::new(state);
        self.states.push(b);
        self.states
            .last()
            .unwrap()
            .deref()
            .downcast_ref::<S>()
            .unwrap()
    }

    /// Returns an already recycled state.
    pub fn get_state<S: 'static>(&self) -> Option<&S> {
        self.states
            .iter()
            .find_map(|s| s.deref().downcast_ref::<S>())
    }

    pub fn scoped(
        &mut self,
        scope: impl Into<ContextScope>,
        builder: impl FnOnce(ViewBuilder<Msg>) -> View<Msg>,
    ) -> ScopedView<Msg> {
        let view = self.context.scoped(scope, |c| builder(Self::new(c)));
        ScopedView::new(view)
    }

    /// Attaches a processor to a View.
    ///
    /// This function recycles a processor with the same type from the current context.
    /// TODO: this function should not leak the type `ProcessorWithSubscription<R>`
    /// TODO: is there any use here in returning the input processor similar to what `use_state()` does?
    pub fn use_input_processor<R>(&mut self, construct: impl FnOnce() -> R)
    where
        R: InputProcessor<In = WindowMessage, Out = Msg> + Subscriber + 'static,
    {
        let r = self.context.recycle_state::<R>();
        let r = r.unwrap_or_else(|| construct().into());

        // need to store a function alongside the processor that converts it from an `Any` to its
        // concrete type, so that it can be converted back to `Any` in the next rendering cycle.
        let record = ProcessorRecord::new(r);
        self.processors.push(record);
    }

    /// Convert the builder into a wrapped view. That is one that contains only one nested view.
    pub fn wrapped(self, nested: ScopedView<Msg>) -> View<Msg> {
        self.combined(iter::once(nested))
    }

    /// Converts the builder into a combined view containing a number of scoped views that are
    /// rendered from back to front.
    pub fn combined(self, nested: impl IntoIterator<Item = ScopedView<Msg>>) -> View<Msg> {
        let container = self.present(Presentation::Empty);
        View::new_combined(container, nested.into_iter().map(|sv| sv.into_view()))
    }

    /// Convert the builder into a (leaf) view that contains only a presentation.
    pub fn present(self, presentation: Presentation) -> View<Msg> {
        View::new(
            presentation,
            self.processors,
            // TODO: may pre-generate with empty ContextPaths to avoid a reallocation here?
            self.states
                .into_iter()
                .map(|s| (ContextPath::new(), s))
                .collect(),
        )
    }
}

impl<Msg> MeasureText for ViewBuilder<Msg> {
    fn measure_text(&self, text: &Text) -> Bounds {
        self.context.measure_text(text)
    }
}
