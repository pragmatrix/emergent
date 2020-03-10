//! An input processor that can activate a subsequent processor.
//!
//! Activation is based on a predicate and a function that generates the processor.
//!
//! The predicate is verified at every change of the states at the processor's context scope and
//! might drop the subsequent processor at any time.

use crate::{InputProcessor, InputState};
use std::marker::PhantomData;

pub struct Activator<R, PF, SP, CF, SC> {
    recognizer: Option<R>,
    predicate: PF,
    constructor: CF,
    pd: PhantomData<(*const SP, *const SC)>,
}

impl<R, PF, SP, CF, SC> Activator<R, PF, SP, CF, SC> {
    pub fn new(predicate: PF, constructor: CF) -> Self
    where
        PF: Fn(&SP) -> bool,
        CF: Fn(&SC) -> R,
    {
        Self {
            recognizer: None,
            predicate,
            constructor,
            pd: PhantomData,
        }
    }
}

impl<R, PF, SP, CF, SC> InputProcessor for Activator<R, PF, SP, CF, SC>
where
    R: InputProcessor,
    PF: Fn(&SP) -> bool,
    CF: Fn(&SC) -> R,
    SP: 'static,
    SC: 'static,
{
    type In = R::In;
    type Out = R::Out;

    fn dispatch(&mut self, input_state: &mut InputState, message: Self::In) -> Option<Self::Out> {
        let state: Option<&mut SP> = input_state.get_state();
        match state {
            Some(sp) if (self.predicate)(sp) => match self.recognizer {
                Some(ref mut r) => r.dispatch(input_state, message),
                None => {
                    let state = input_state.get_state()?;
                    let r = (self.constructor)(state);
                    self.recognizer = Some(r);
                    self.recognizer
                        .as_mut()
                        .unwrap()
                        .dispatch(input_state, message)
                }
            },
            _ => {
                self.recognizer = None;
                None
            }
        }
    }
}

/*
impl<R, PF, SP, CF, SC> Subscriber for Activator<R, PF, SP, CF, SC> {
    fn subscriptions(&self) -> Subscriptions {
        Subscriptions::from_iter([Subscription::StateChanged].iter())
    }
}
*/
