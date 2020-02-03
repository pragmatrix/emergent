use crate::transaction;
use crate::InputState;
use emergent_drawing::ReplaceWith;
use std::any::{type_name, TypeId};
use std::marker::PhantomData;
use std::mem;

/// A trait to define input processors.
///
/// Input processors are entities that process input messages and expose output events.
///
/// While processing events, they can access and modify state through the `InputState` instance.
pub trait InputProcessor {
    /// The input message.
    type In;
    /// The output event of input processor.
    type Out;

    fn dispatch(&mut self, input_state: &mut InputState, message: Self::In) -> Option<Self::Out>;

    /// Map the resulting event to another.
    ///
    /// TODO: may call this function map_out()?
    fn map<F, To>(self, f: F) -> Map<Self, F>
    where
        F: Fn(Self::Out) -> Option<To>,
        Self: Sized,
    {
        Map {
            processor: self,
            map_event: f,
        }
    }

    /// Apply the resulting event to another function that can modify another view state and return another event.
    fn apply<To, F, S>(self, f: F) -> Apply<Self, F, S>
    where
        F: Fn(S, Self::Out) -> (S, Option<To>),
        Self: Sized,
    {
        Apply {
            recognizer: self,
            apply: f,
            pd: PhantomData,
        }
    }

    /// Apply the resulting event to another function that can modify another view state and return another event.
    fn apply_mut<To, F, S>(self, f: F) -> ApplyMut<Self, F, S>
    where
        F: Fn(Self::Out, &mut S) -> Option<To>,
        Self: Sized,
    {
        ApplyMut {
            recognizer: self,
            apply: f,
            pd: PhantomData,
        }
    }

    /// Optionally activates a transaction in response to an event.
    fn activate<S, I, U, Out>(self, initiator: I) -> Activate<Self, S, I, U, Out>
    where
        I: Fn(Self::Out, &mut S) -> transaction::InitialResponse<U, Out>,
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
    processor: R,
    map_event: F,
}

impl<To, R, F> InputProcessor for Map<R, F>
where
    R: InputProcessor,
    F: Fn(R::Out) -> Option<To>,
{
    type In = R::In;
    type Out = To;

    fn dispatch(&mut self, input_state: &mut InputState, message: R::In) -> Option<Self::Out> {
        let event = self.processor.dispatch(input_state, message);
        event.and_then(&self.map_event)
    }
}

pub struct Apply<R, F, S> {
    recognizer: R,
    apply: F,
    pd: PhantomData<*const S>,
}

impl<To, R, F, S: 'static> InputProcessor for Apply<R, F, S>
where
    R: InputProcessor,
    F: Fn(S, R::Out) -> (S, Option<To>),
{
    type In = R::In;
    type Out = To;

    fn dispatch(&mut self, input_state: &mut InputState, message: Self::In) -> Option<To> {
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

pub struct ApplyMut<R, F, S> {
    recognizer: R,
    apply: F,
    pd: PhantomData<*const S>,
}

impl<To, R, F, S: 'static> InputProcessor for ApplyMut<R, F, S>
where
    R: InputProcessor,
    F: Fn(R::Out, &mut S) -> Option<To>,
{
    type In = R::In;
    type Out = To;

    fn dispatch(&mut self, input_state: &mut InputState, message: Self::In) -> Option<Self::Out> {
        let e = self.recognizer.dispatch(input_state, message)?;
        let state: &mut S = input_state.get_state()?;
        (self.apply)(e, state)
    }
}

pub struct Activate<R, S, I, U, Out>
where
    R: InputProcessor,
    I: Fn(R::Out, &mut S) -> transaction::InitialResponse<U, Out>,
{
    recognizer: R,
    initiator: I,
    transaction: Option<(S, U)>,
}

impl<R, In, S, I, U, Out> InputProcessor for Activate<R, S, I, U, Out>
where
    R: InputProcessor<In = In>,
    S: 'static + Clone, // Clone to support rollback
    I: Fn(R::Out, &mut S) -> transaction::InitialResponse<U, Out>,
    U: FnMut(R::Out, &mut S) -> transaction::UpdateResponse<Out>,
{
    type In = In;
    type Out = Out;

    fn dispatch(&mut self, input_state: &mut InputState, message: In) -> Option<Self::Out> {
        use transaction::{InitialAction::*, UpdateAction::*};

        let e = self.recognizer.dispatch(input_state, message);

        if e.is_none() {
            if self.transaction.is_some() && input_state.get_state::<S>().is_none() {
                warn!(
                    "state {} vanished, but may reappear before the transaction continues",
                    type_name::<S>(),
                )
            }
            return None;
        }
        let e = e.unwrap();

        let state = input_state.get_state();
        if state.is_none() {
            info!(
                "state {} {:?} vanished, cleaning up, {} got ignored",
                type_name::<S>(),
                TypeId::of::<S>(),
                type_name::<R::Out>()
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
