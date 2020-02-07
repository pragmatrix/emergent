use crate::InputState;
use std::marker::PhantomData;

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
    fn apply<To, F, S>(self, f: F) -> Apply<Self, F, To, S>
    where
        F: Fn(Self::Out, &mut S) -> Option<To>,
        Self: Sized,
    {
        Apply {
            recognizer: self,
            apply: f,
            pd: PhantomData,
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

pub struct Apply<R, F, To, S> {
    recognizer: R,
    apply: F,
    pd: PhantomData<(*const S, *const To)>,
}

impl<To, R, F, S: 'static> InputProcessor for Apply<R, F, To, S>
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
