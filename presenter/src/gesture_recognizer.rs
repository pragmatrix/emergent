use crate::InputState;
use emergent_drawing::ReplaceWith;
use emergent_ui::WindowMessage;
use std::marker::PhantomData;

/// A trait to define gesture recognizers.
///
/// Gesture recognizers are persisting and are updated with
/// each WindowMessage.
///
/// Their lifetime is bound to the context scope they are rendered at first.

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
        Map::new(self, f)
    }

    /// Apply the resulting event to another a function that can modify another view state and return another event.
    /// TODO: use a kind of typed ContextPath, a StateReference?
    fn apply<To, F, S>(self, f: F) -> Apply<Self, F, S>
    where
        F: Fn(S, Self::Event) -> (S, Option<To>),
        Self: Sized,
    {
        Apply::new(self, f)
    }
}

pub struct Map<R, F> {
    recognizer: R,
    map_event: F,
}

impl<R, F> Map<R, F> {
    pub fn new(recognizer: R, map_event: F) -> Self {
        Self {
            recognizer,
            map_event,
        }
    }
}

impl<From, To, R, F> GestureRecognizer for Map<R, F>
where
    R: GestureRecognizer<Event = From>,
    F: Fn(From) -> Option<To>,
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

impl<R, F, S> Apply<R, F, S> {
    pub fn new(recognizer: R, apply: F) -> Self {
        Self {
            recognizer,
            apply,
            pd: PhantomData,
        }
    }
}

impl<From, To, R, F, S: 'static> GestureRecognizer for Apply<R, F, S>
where
    R: GestureRecognizer<Event = From>,
    F: Fn(S, From) -> (S, Option<To>),
{
    type Event = To;

    fn update_with_input_state(
        &mut self,
        context: InputState,
        message: WindowMessage,
    ) -> (InputState, Option<Self::Event>) {
        let (mut input_state, e) = self.recognizer.update_with_input_state(context, message);

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
