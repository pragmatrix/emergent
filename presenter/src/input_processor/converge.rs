//! An input processor that converges a move / pan gesture to specific position.
//!
//! The convergence is defined by an easing function.
//!
//! This processor is meant to be combined with a processor that preserves momentum, so that the drift
//! converges to a constrained ending position.
//!
//! Notes:
//! - Only the drift phase of the processor that preserves momentum is considered.
//! - We don't need to subscribe to ticks here, because the one that sends the events already does.

use crate::input_processor::momentum::Phase;
use crate::input_processor::transaction::Transaction;
use crate::input_processor::{Subscriber, Subscriptions};
use crate::{InputProcessor, InputState};
use emergent_drawing::{scalar, Point};
use std::marker::PhantomData;
use std::time::{Duration, Instant};

pub struct Converge<P, TF, S> {
    processor: P,
    get_target: TF,
    duration: Duration,
    easing: fn(scalar) -> scalar,
    state: State,
    pd: PhantomData<*const S>,
}

enum State {
    Idle,
    Drifting { start_t: Instant },
}

pub trait ConvergeTo {
    fn converge_to<TF, S>(
        self,
        get_target: TF,
        duration: Duration,
        easing: fn(scalar) -> scalar,
    ) -> Converge<Self, TF, S>
    where
        TF: Fn(&S) -> Point,
        Self: Sized,
    {
        Converge {
            processor: self,
            get_target,
            duration,
            easing,
            state: State::Idle,
            pd: PhantomData,
        }
    }
}

impl<T> ConvergeTo for T where T: InputProcessor {}

impl<P, TF, S> InputProcessor for Converge<P, TF, S>
where
    P: InputProcessor<Out = Transaction<(Point, Phase)>> + Sized,
    TF: Fn(&S) -> Point,
    S: 'static,
{
    type In = P::In;
    type Out = Transaction<(Point, Phase)>;

    fn dispatch(&mut self, input_state: &mut InputState, message: Self::In) -> Option<Self::Out> {
        let e = self.processor.dispatch(input_state, message)?;
        use Transaction::*;

        match e {
            Update((ref current_pos, Phase::Drifting)) => match self.state {
                State::Idle => {
                    self.state = State::Drifting {
                        start_t: input_state.time(),
                    };
                    e
                }
                State::Drifting { start_t } => {
                    let dt = input_state.time() - start_t;
                    let t = dt.as_secs_f64() / self.duration.as_secs_f64();
                    // note: target may be moving, so we ask for it each time.
                    let state = input_state.get_state::<S>().unwrap();
                    let target = (self.get_target)(state);
                    let f = (self.easing)(t);
                    let pt = current_pos.to_vector() * (1.0 - f) + target.to_vector() * f;
                    Update((Point::from(pt), Phase::Drifting))
                }
            },
            Commit((_, Phase::Drifting)) => {
                let state = input_state.get_state::<S>().unwrap();
                let target = (self.get_target)(state);
                Commit((target, Phase::Drifting))
            }
            e => {
                self.state = State::Idle;
                e
            }
        }
        .into()
    }
}

impl<P, TF, S> Subscriber for Converge<P, TF, S>
where
    P: Subscriber,
{
    fn subscriptions(&self) -> Subscriptions {
        self.processor.subscriptions()
    }
}
