use crate::recognizer::{AutoSubscribe, Recognizer};
use crate::{
    AreaHitTest, Context, GestureRecognizer, InputState, RecognizerRecord, ScopedStore, Support,
    View,
};
use emergent_presentation::Presentation;
use emergent_ui::{FrameLayout, WindowMessage};
use std::any::TypeId;
use std::collections::HashSet;
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

    pub fn dispatch_window_message(&mut self, msg: WindowMessage) -> Vec<Msg>
    where
        Msg: 'static,
    {
        let position = msg.state.cursor_position().unwrap();
        debug!("position for hit testing {:?}", position);

        let hits = {
            let presentation = self.presentation();
            presentation.area_hit_test(position, Vec::new(), self.support())
        };
        debug!("hits: {:?}", hits);

        let presentation_scope_hits: HashSet<_> = hits.into_iter().map(|(s, _p)| s).collect();

        debug!(
            "hit at presentations: {:?}, event: {:?}, state: {:?}",
            presentation_scope_hits, msg.event, msg.state
        );

        // TODO: what to do about the relative hit positions?

        let store = &mut self.store;
        self.recognizers
            .iter_mut()
            // filter_map because we need mutable access.
            .filter_map(|r| {
                if r.subscriptions().wants_event(&msg.event)
                    || presentation_scope_hits.contains(r.presentation_path())
                {
                    Some(r)
                } else {
                    None
                }
            })
            .map(|recognizer| {
                let c = recognizer.context_path().clone();

                debug!("recognizer for hit at context: {:?}", c);
                let states = store.remove_states_at(&c);
                debug!(
                    "states at {:?}: {} {:?}",
                    c,
                    states.len(),
                    states
                        .iter()
                        .map(|s| s.deref().type_id())
                        .collect::<Vec<TypeId>>()
                );

                // process automatic subscriptions _before_ dispatching the message into the recognizer, so that it
                // can veto.

                msg.event.auto_subscribe(recognizer.subscriptions());

                let mut input_state =
                    InputState::new(c.clone(), recognizer.subscriptions().clone(), states);
                let msg = recognizer.dispatch(&mut input_state, msg.clone());
                let (new_subscriptions, new_context_states) = input_state.into_states();
                *recognizer.subscriptions() = new_subscriptions;
                store.extend_states_at(&c, new_context_states);

                msg
            })
            .flatten()
            .collect()
    }
}
