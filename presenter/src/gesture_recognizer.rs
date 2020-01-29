use crate::transaction;
use crate::InputState;
use emergent_drawing::ReplaceWith;
use emergent_ui::WindowMessage;
use std::any::{type_name, TypeId};
use std::marker::PhantomData;
use std::mem;

/// A trait to define gesture recognizers.
///
/// Gesture recognizers are persisting and are updated with
/// each WindowMessage.
pub trait GestureRecognizer {
    /// The resulting event of the gesture recognizer.
    type Event;

    fn dispatch(&mut self, context: &mut InputState, message: WindowMessage)
        -> Option<Self::Event>;

    /// Map the resulting event to another.
    ///
    /// TODO: may call this function map_out()?
    fn map<F, To>(self, f: F) -> Map<Self, F>
    where
        F: Fn(Self::Event) -> Option<To>,
        Self: Sized,
    {
        Map {
            recognizer: self,
            map_event: f,
        }
    }

    /// Apply the resulting event to another function that can modify another view state and return another event.
    fn apply<To, F, S>(self, f: F) -> Apply<Self, F, S>
    where
        F: Fn(S, Self::Event) -> (S, Option<To>),
        Self: Sized,
    {
        Apply {
            recognizer: self,
            apply: f,
            pd: PhantomData,
        }
    }

    /// Optionally activates a transaction in response to an event.
    fn activate<S, I, U, Out>(self, initiator: I) -> Activate<Self, S, I, U, Out>
    where
        I: Fn(Self::Event, &mut S) -> transaction::InitialResponse<U, Out>,
        Self: Sized,
    {
        Activate {
            recognizer: self,
            initiator,
            transaction: None,
        }
    }
}

pub struct Map<R, F> {
    recognizer: R,
    map_event: F,
}

impl<To, R, F> GestureRecognizer for Map<R, F>
where
    R: GestureRecognizer,
    F: Fn(R::Event) -> Option<To>,
{
    type Event = To;

    fn dispatch(
        &mut self,
        input_state: &mut InputState,
        message: WindowMessage,
    ) -> Option<Self::Event> {
        let event = self.recognizer.dispatch(input_state, message);
        event.and_then(&self.map_event)
    }
}

pub struct Apply<R, F, S> {
    recognizer: R,
    apply: F,
    pd: PhantomData<*const S>,
}

impl<To, R, F, S: 'static> GestureRecognizer for Apply<R, F, S>
where
    R: GestureRecognizer,
    F: Fn(S, R::Event) -> (S, Option<To>),
{
    type Event = To;

    fn dispatch(
        &mut self,
        input_state: &mut InputState,
        message: WindowMessage,
    ) -> Option<Self::Event> {
        let e = self.recognizer.dispatch(input_state, message);

        if let Some(e) = e {
            let mut to_r = None;
            input_state.modify(|s: &mut S| {
                s.replace_with(|s| {
                    let (s, t) = (self.apply)(s, e);
                    to_r = t;
                    s
                })
            });
            return to_r;
        }

        None
    }
}

pub struct Activate<R, S, I, U, Out>
where
    R: GestureRecognizer,
    I: Fn(R::Event, &mut S) -> transaction::InitialResponse<U, Out>,
{
    recognizer: R,
    initiator: I,
    transaction: Option<(S, U)>,
}

impl<R, S, I, U, Out> GestureRecognizer for Activate<R, S, I, U, Out>
where
    R: GestureRecognizer,
    S: 'static + Clone, // Clone to support rollback
    I: Fn(R::Event, &mut S) -> transaction::InitialResponse<U, Out>,
    U: FnMut(R::Event, &mut S) -> transaction::UpdateResponse<Out>,
{
    type Event = Out;

    fn dispatch(
        &mut self,
        input_state: &mut InputState,
        message: WindowMessage,
    ) -> Option<Self::Event> {
        use transaction::{InitialAction::*, UpdateAction::*};

        let e = self.recognizer.dispatch(input_state, message);

        if e.is_none() {
            if self.transaction.is_some() && input_state.get_mut::<S>().is_none() {
                warn!(
                    "view state {} vanished, but may reappear before the transaction continues",
                    type_name::<S>(),
                )
            }
            return None;
        }
        let e = e.unwrap();

        let state = input_state.get_mut();
        if state.is_none() {
            info!(
                "view state {} {:?} vanished, cleaning up, {} got ignored",
                type_name::<S>(),
                TypeId::of::<S>(),
                type_name::<R::Event>()
            );
            self.transaction = None;
            return None;
        }
        let state = state.unwrap();

        match &mut self.transaction {
            None => {
                let response = (self.initiator)(e, state);
                match response.action {
                    Neglect => {}
                    Begin(u) => {
                        self.transaction = Some((state.clone(), u));
                    }
                }
                response.event
            }
            Some((rollback_state, u)) => {
                let response = u(e, state);
                match response.action {
                    Sustain => {}
                    Commit => {
                        self.transaction = None;
                    }
                    Rollback => {
                        mem::swap(state, rollback_state);
                        self.transaction = None
                    }
                }

                response.event
            }
        }
    }
}
