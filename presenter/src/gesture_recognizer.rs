use crate::transaction;
use crate::{InputState, Transaction};
use emergent_drawing::ReplaceWith;
use emergent_ui::WindowMessage;
use std::any::type_name;
use std::marker::PhantomData;
use std::mem;

/// A trait to define gesture recognizers.
///
/// Gesture recognizers are persisting and are updated with
/// each WindowMessage.
pub trait GestureRecognizer {
    type Event;

    fn update_with_input_state(
        &mut self,
        context: InputState,
        message: WindowMessage,
    ) -> (InputState, Option<Self::Event>) {
        (context, self.update(message))
    }

    fn update(&mut self, _message: WindowMessage) -> Option<Self::Event> {
        None
    }

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

    /// Activates an input transaction in response to an event.
    fn activate<T>(self) -> Activate<Self, T>
    where
        T: Transaction,
        Self: Sized,
    {
        Activate {
            recognizer: self,
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

    fn update_with_input_state(
        &mut self,
        context: InputState,
        message: WindowMessage,
    ) -> (InputState, Option<Self::Event>) {
        let (context, event) = self.recognizer.update_with_input_state(context, message);
        (context, event.and_then(&self.map_event))
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

    fn update_with_input_state(
        &mut self,
        input_state: InputState,
        message: WindowMessage,
    ) -> (InputState, Option<Self::Event>) {
        let (mut input_state, e) = self
            .recognizer
            .update_with_input_state(input_state, message);

        if let Some(e) = e {
            let mut to_r = None;
            input_state.modify(|s: &mut S| {
                s.replace_with(|s| {
                    let (s, t) = (self.apply)(s, e);
                    to_r = t;
                    s
                })
            });
            return (input_state, to_r);
        }

        (input_state, None)
    }
}

pub struct Activate<R, T>
where
    T: Transaction,
{
    recognizer: R,
    transaction: Option<(T::ViewState, T)>,
}

impl<R, T> GestureRecognizer for Activate<R, T>
where
    R: GestureRecognizer,
    T: Transaction<InputEvent = R::Event>,
    T::ViewState: 'static + Clone, // Clone to support rollback
{
    type Event = T::OutputEvent;

    fn update_with_input_state(
        &mut self,
        input_state: InputState,
        message: WindowMessage,
    ) -> (InputState, Option<Self::Event>) {
        use transaction::{InitialAction::*, UpdateAction::*};

        let (mut input_state, e) = self
            .recognizer
            .update_with_input_state(input_state, message);
        if e.is_none() {
            if self.transaction.is_some() && input_state.get_mut::<T::ViewState>().is_none() {
                warn!(
                    "view state {} vanished, but may reappear before {} continues",
                    type_name::<T::ViewState>(),
                    type_name::<T>()
                )
            }
            return (input_state, None);
        }
        let e = e.unwrap();

        let state = input_state.get_mut();
        if state.is_none() {
            info!(
                "view state {} for {} vanished, cleaning up, {} got ignored",
                type_name::<T::ViewState>(),
                type_name::<T>(),
                type_name::<R::Event>()
            );
            self.transaction = None;
            return (input_state, None);
        }
        let state = state.unwrap();

        match &mut self.transaction {
            None => {
                let response = T::try_begin(e, state);
                match response.action {
                    Neglect => {}
                    Begin(t) => {
                        self.transaction = Some((state.clone(), t));
                    }
                }
                (input_state, response.event)
            }
            Some(t) => {
                let (rollback_state, t) = t;
                let response = t.update(e, state);
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

                (input_state, response.event)
            }
        }
    }
}
