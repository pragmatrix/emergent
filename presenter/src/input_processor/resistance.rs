//! Apply resistance to a move gesture.

use crate::input_processor::transaction::Transaction;
use crate::{InputProcessor, InputState};
use emergent_drawing::{Point, Vector};
use std::marker::PhantomData;

pub trait WithResistance {
    fn with_resistance<F, State>(self, get_resistance_vector: F) -> Resistance<Self, F, State>
    where
        F: Fn(Point, &State) -> Vector,
        Self: Sized,
    {
        // TODO: implement with InputProcessor::map_with_state() ? something like that.
        Resistance {
            processor: self,
            get_resistance_vector,
            pd: PhantomData,
        }
    }
}

impl<T> WithResistance for T where T: InputProcessor<Out = Transaction<Point>> {}

pub struct Resistance<P, F, State> {
    processor: P,
    get_resistance_vector: F,
    pd: PhantomData<*const State>,
}

impl<P, F, State> InputProcessor for Resistance<P, F, State>
where
    P: InputProcessor<Out = Transaction<Point>>,
    F: Fn(Point, &State) -> Vector,
    State: 'static,
{
    type In = P::In;
    type Out = P::Out;

    fn dispatch(&mut self, input_state: &mut InputState, message: Self::In) -> Option<Self::Out> {
        let e = self.processor.dispatch(input_state, message)?;
        let state: &State = input_state.get_state()?;
        e.map(|p| p + (self.get_resistance_vector)(p, state)).into()
    }
}
