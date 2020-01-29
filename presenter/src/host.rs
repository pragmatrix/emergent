use crate::{
    Context, GestureRecognizer, InputState, PresentationPath, RecognizerRecord, ScopedStore,
    Support, View,
};
use emergent_drawing::Point;
use emergent_presentation::Presentation;
use emergent_ui::{FrameLayout, WindowMessage};
use std::any::TypeId;
use std::mem;
use std::ops::Deref;
use std::rc::Rc;

pub struct Host<Msg> {
    support: Rc<Support>,

    presentation: Presentation,

    /// The recognizers that are active.
    recognizers: Vec<RecognizerRecord<Msg>>,

    /// The store of all captured states of the context scopes.
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

    pub fn present(&mut self, boundary: FrameLayout, present: impl FnOnce(Context) -> View<Msg>)
    where
        Msg: 'static,
    {
        // get all the states from the previous run.
        let store = mem::replace(&mut self.store, ScopedStore::default());
        // move all recognizers into the store so that they can get recycled, too.
        let recognizer_store =
            ScopedStore::from_values(self.recognizers.drain(..).map(|r| r.into_scoped_state()));
        info!("recognizers at []: {:?}", recognizer_store.states);
        let store = store.merged(recognizer_store);

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
        debug!("event: {:?}", msg.event);
        debug!("msg state: {:?}", msg.state);

        // TODO: what about multiple hits?

        let r = self
            .recognizers
            .iter_mut()
            .find(|r| *r.presentation_path() == presentation_path)?;

        let c = r.context_path().clone();

        debug!("recognizer for hit at context: {:?}", c);
        let states = self.store.remove_states_at(&c);
        debug!(
            "states at {:?}: {} {:?}",
            c,
            states.len(),
            states
                .iter()
                .map(|s| s.deref().type_id())
                .collect::<Vec<TypeId>>()
        );
        let mut input_state = InputState::new(c.clone(), states);
        let msg = r.dispatch(&mut input_state, msg);
        let new_states = input_state.into_states();
        self.store.extend_states_at(&c, new_states);

        msg
    }
}
