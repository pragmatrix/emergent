use crate::input_processor;
use crate::ContextPath;
use std::any;
use std::any::{Any, TypeId};
use std::ops::Deref;
use std::time::Instant;

/// The `InputState` maintains all the state that may be accessed and modified while input is being processed by one
/// single input processor.
pub struct InputState {
    /// The processor's context. This is used for resolving states.
    processor_context: ContextPath,
    /// The time the input event was sent.
    time: Instant,
    /// The states available to be modified by the input processor.
    /// There should be a very limited amount of states per context path, so a vector is fine for doing
    /// lookups.
    states: Vec<Box<dyn Any>>,
}

impl InputState {
    pub fn new(
        processor_context: ContextPath,
        time: Instant,
        subscriptions: input_processor::Subscriptions,
        states: impl IntoIterator<Item = Box<dyn Any>>,
    ) -> Self {
        Self {
            processor_context,
            time,
            states: states.into_iter().collect(),
        }
    }

    pub fn time(&self) -> Instant {
        self.time
    }

    //
    // context associated state
    //

    /// Modify a typed state record.
    pub fn modify<S: 'static>(&mut self, f: impl FnOnce(&mut S)) {
        let type_id = TypeId::of::<S>();

        // TODO: find  way to do this with iter_mut().find(),
        //       ... there are conflicting lifetime requirements I do not understand.

        let states = &mut self.states;

        for i in 0..states.len() {
            if states[i].deref().type_id() == type_id {
                let r = states[i].downcast_mut::<S>().unwrap();
                f(r);
                return;
            }
        }

        panic!(
            "found no state {} in {:?}",
            any::type_name::<S>(),
            self.processor_context
        )
    }

    /// Return a mutable reference to a typed state record.
    pub fn get_state<S: 'static>(&mut self) -> Option<&mut S> {
        let type_id = TypeId::of::<S>();
        let states = &mut self.states;

        states
            .iter_mut()
            .find(|s| s.deref().deref().type_id() == type_id)
            .map(|s| s.downcast_mut::<S>().unwrap())
    }

    pub fn into_states(self) -> Vec<Box<dyn Any>> {
        self.states
    }
}
