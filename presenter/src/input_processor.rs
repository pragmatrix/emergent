use crate::InputState;
use std::marker::PhantomData;
use transaction::Transaction;

pub mod animator;
pub use animator::Animator;

pub mod converge;

pub mod easing;

pub mod transaction;

pub mod momentum;

pub mod move_threshold;

pub mod pan;
pub use pan::Pan;

pub mod resistance;

mod subscriptions;
pub use subscriptions::*;

pub mod tap;
pub use tap::Tap;

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
    /// TODO: with state support, this could be generalized so that it can replace resistance or at least can be used
    /// for that.
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

    /// Provide a map function at beginning of a transaction event.
    fn map_begin<F, State, FM, DataIn, DataOut>(
        self,
        map_begin: F,
    ) -> MapBegin<Self, F, State, FM, DataIn, DataOut>
    where
        Self: InputProcessor<Out = Transaction<DataIn>> + Sized,
        F: Fn(DataIn, &State) -> Option<FM>,
        FM: Fn(DataIn) -> DataOut,
        State: 'static,
        DataIn: Clone,
    {
        MapBegin {
            processor: self,
            map_begin,
            map_transaction: None,
            pd: PhantomData,
        }
    }

    /// Apply the resulting event to another function that can modify another view state and return another event.
    fn apply<To, F, S>(self, f: F) -> Apply<Self, F, To, S>
    where
        F: Fn(Self::Out, &mut S) -> Option<To>,
        Self: Sized,
    {
        Apply {
            processor: self,
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
    processor: R,
    apply: F,
    pd: PhantomData<(*const S, *const To)>,
}

impl<To, P, F, S: 'static> InputProcessor for Apply<P, F, To, S>
where
    P: InputProcessor,
    F: Fn(P::Out, &mut S) -> Option<To>,
{
    type In = P::In;
    type Out = To;

    fn dispatch(&mut self, input_state: &mut InputState, message: Self::In) -> Option<Self::Out> {
        let e = self.processor.dispatch(input_state, message)?;
        let state: &mut S = input_state.get_state()?;
        (self.apply)(e, state)
    }
}

pub struct MapBegin<P, F, State, FM, DataIn, DataOut> {
    processor: P,
    map_begin: F,
    map_transaction: Option<FM>,
    pd: PhantomData<(*const FM, *const State, *const DataIn, *const DataOut)>,
}

impl<P, F, State, FM, DataIn, DataOut> InputProcessor for MapBegin<P, F, State, FM, DataIn, DataOut>
where
    P: InputProcessor<Out = Transaction<DataIn>>,
    F: Fn(DataIn, &State) -> Option<FM>,
    FM: Fn(DataIn) -> DataOut,
    State: 'static,
    DataIn: Clone,
{
    type In = P::In;
    type Out = Transaction<DataOut>;

    fn dispatch(&mut self, input_state: &mut InputState, message: Self::In) -> Option<Self::Out> {
        let e = self.processor.dispatch(input_state, message)?;
        let state = input_state.get_state::<State>()?;
        use Transaction::*;
        match e {
            Begin(d) => {
                let t = (self.map_begin)(d.clone(), state)?;
                let e = Begin(t(d));
                self.map_transaction = Some(t);
                e
            }
            Update(d) => Update(self.map_transaction.as_ref()?(d)),
            Commit(d) => {
                let e = Commit(self.map_transaction.as_ref()?(d));
                self.map_transaction = None;
                e
            }
            Rollback(d) => {
                let e = Rollback(self.map_transaction.as_ref()?(d));
                self.map_transaction = None;
                e
            }
        }
        .into()
    }
}
