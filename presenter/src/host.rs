use crate::input_processor::{AutoSubscribe, Subscription};
use crate::processor::Processor;
use crate::{
    AreaHitTest, Context, InputProcessor, InputState, ProcessorRecord, ScopedStore, Support, View,
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

    /// The input processors that are active.
    processors: Vec<ProcessorRecord<Msg>>,

    /// The store of all captured states of the context scopes.
    store: ScopedStore,
}

impl<Msg> Host<Msg> {
    pub fn new(support: Support) -> Host<Msg> {
        Host {
            support: Rc::new(support),
            presentation: Default::default(),
            processors: Default::default(),
            store: ScopedStore::default(),
        }
    }

    pub fn present(&mut self, boundary: FrameLayout, present: impl FnOnce(Context) -> View<Msg>)
    where
        Msg: 'static,
    {
        // get all the states from the previous run.
        let store = mem::replace(&mut self.store, ScopedStore::default());
        // move all processors into the store so that they can get recycled, too.
        let processor_store =
            ScopedStore::from_values(self.processors.drain(..).map(|r| r.into_scoped_state()));
        info!("processors at []: {:?}", processor_store.states);
        let store = store.merged(processor_store);

        let context = Context::new(self.support.clone(), boundary, store);
        let (presentation, processors, states) = present(context).destructure();

        self.presentation = presentation;
        self.processors = processors;
        self.store = ScopedStore::from_values(states);
    }

    pub fn support(&self) -> &Support {
        self.support.deref()
    }

    pub fn presentation(&self) -> &Presentation {
        &self.presentation
    }

    // TODO: don't need mut self here.
    pub fn needs_ticks(&mut self) -> bool {
        self.processors
            .iter_mut()
            .any(|r| r.subscriptions().contains(Subscription::Ticks))
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
        self.processors
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
            .map(|processor| {
                let c = processor.context_path().clone();

                debug!("processor for hit at context: {:?}", c);
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

                // process automatic subscriptions _before_ dispatching the message into the processor, so that it
                // can veto.

                msg.event.auto_subscribe(processor.subscriptions());

                let mut input_state = InputState::new(
                    c.clone(),
                    msg.time,
                    processor.subscriptions().clone(),
                    states,
                );
                let msg = processor.dispatch(&mut input_state, msg.clone());
                let (new_subscriptions, new_context_states) = input_state.into_states();
                *processor.subscriptions() = new_subscriptions;
                store.extend_states_at(&c, new_context_states);

                msg
            })
            .flatten()
            .collect()
    }
}
