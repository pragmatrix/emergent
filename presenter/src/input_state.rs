use crate::ContextPath;
use std::any;
use std::any::{Any, TypeId};
use std::ops::Deref;

/// Represents all the state that is modified while input is being processed.
pub struct InputState {
    /// The recognizer's path. This is used for resolving states.
    recognizer_context: ContextPath,
    /// The states available to be modified by the gesture recognizer.
    /// Usually there are not many state per context path, so a vector should be fine for doing lookups.
    states: Vec<Box<dyn Any>>,
}

impl InputState {
    // TODO: is it even worth pushing _all_ states into a hashmap here?
    pub fn new(
        recognizer_context: ContextPath,
        states: impl IntoIterator<Item = Box<dyn Any>>,
    ) -> Self {
        Self {
            recognizer_context,
            states: states.into_iter().collect(),
        }
    }

    pub fn into_states(self) -> Vec<Box<dyn Any>> {
        self.states
    }

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
            self.recognizer_context
        )
    }

    /// Return a mutable reference to a typed state record.
    pub fn get_mut<S: 'static>(&mut self) -> Option<&mut S> {
        let type_id = TypeId::of::<S>();
        let states = &mut self.states;

        states
            .iter_mut()
            .find(|s| s.deref().deref().type_id() == type_id)
            .map(|s| s.downcast_mut::<S>().unwrap())
    }
}
