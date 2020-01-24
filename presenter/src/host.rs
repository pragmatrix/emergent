use crate::{
    Context, ContextPath, GestureRecognizer, InputState, PresentationPath, ScopedStore, Support,
    View,
};
use emergent_drawing::Point;
use emergent_presentation::Presentation;
use emergent_ui::{FrameLayout, WindowMessage};
use std::mem;
use std::ops::Deref;
use std::rc::Rc;

type RecognizerRecord<Msg> = (
    PresentationPath,
    ContextPath,
    Box<dyn GestureRecognizer<Event = Msg>>,
);

pub struct Host<Msg> {
    support: Rc<Support>,

    presentation: Presentation,

    /// The recognizers that are active.
    recognizers: Vec<RecognizerRecord<Msg>>,

    /// The store of all collected states of the context scopes.
    store: ScopedStore,
}

impl<Msg> Host<Msg> {
    pub fn new(support: Support) -> Host<Msg> {
        Host {
            support: Rc::new(support),
            presentation: Default::default(),
            recognizers: Default::default(),
            store: ScopedStore::default(),
        }
    }

    pub fn present(&mut self, boundary: FrameLayout, present: impl FnOnce(Context) -> View<Msg>) {
        let store = mem::replace(&mut self.store, ScopedStore::default());
        let context = Context::new(self.support.clone(), boundary, store);
        let (presentation, recognizers, states) = present(context).destructure();

        self.presentation = presentation;
        self.recognizers = recognizers;
        self.store = ScopedStore::from_values(states);
    }

    pub fn support(&self) -> &Support {
        self.support.deref()
    }

    pub fn presentation(&self) -> &Presentation {
        &self.presentation
    }

    /// Dispatches mouse input to a gesture recognizer and return a Msg if it produces one.
    pub fn dispatch_mouse_input(
        &mut self,
        (presentation_path, _point): (PresentationPath, Point),
        msg: WindowMessage,
    ) -> Option<Msg>
    where
        Msg: 'static,
    {
        debug!("hit at presentation: {:?}", presentation_path);

        // TODO: what about multiple hits?

        let (c, r) = self
            .recognizers
            .iter_mut()
            .find(|(p, _c, _r)| *p == presentation_path)
            .map(|(_p, c, r)| (&*c, r.as_mut()))?;

        debug!("recognizer for hit at context: {:?}", c);
        let states = self.store.remove_states_at(&c);
        debug!("states at {:?}: {}", c, states.len());
        let input_state = InputState::new(c.clone(), states);
        let (input_state, msg) = r.update_with_input_state(input_state, msg);
        let new_states = input_state.into_states();
        self.store.extend_states_at(&c, new_states);

        msg
    }
}
