//! A recognizer that converges a move / pan gesture to specific position.
//!
//! The convergence is defined by an easing function.
//!
//! This recognizer is meant to be combined with a recognizer that preserves momentum, so that the drift
//! converges to a constrained ending position.
//!
//! Notes:
//! - Only the drift phase of the processor that preserves momentum is considered.
//! - We don't need to subscribe to ticks here, because the one that sends the events already does.

use crate::recognizer::momentum::Phase;
use crate::recognizer::transaction::AbsolutePos;
use crate::recognizer::transaction::Transaction;
use crate::{InputProcessor, InputState};
use emergent_drawing::{scalar, Point};
use emergent_ui::WindowMessage;
use std::marker::PhantomData;
use std::time::{Duration, Instant};

pub struct Converge<P, TF, S> {
    processor: P,
    get_target: TF,
    easing: fn(scalar) -> scalar,
    state: State,
    pd: PhantomData<*const S>,
}

enum State {
    Idle,
    Drifting {
        start_t: Instant,
        duration: Duration,
    },
}

pub trait ConvergeTo {
    fn converge_to<TF, S>(
        self,
        get_target: TF,
        easing: fn(scalar) -> scalar,
    ) -> Converge<Self, TF, S>
    where
        TF: Fn(&S) -> Point,
        Self: Sized,
    {
        Converge {
            processor: self,
            get_target,
            easing,
            state: State::Idle,
            pd: PhantomData,
        }
    }
}

impl<T> ConvergeTo for T where T: InputProcessor<In = WindowMessage> {}

impl<P, Data, TF, S> InputProcessor for Converge<P, TF, S>
where
    P: InputProcessor<In = WindowMessage, Out = Transaction<(Data, Phase)>> + Sized,
    TF: Fn(&S) -> Point,
    S: 'static,
    Transaction<(Data, Phase)>: AbsolutePos,
    Data: Clone,
{
    type In = WindowMessage;
    type Out = Transaction<(Data, Phase)>;

    fn dispatch(&mut self, input_state: &mut InputState, message: Self::In) -> Option<Self::Out> {
        let message_time = message.time;
        let e = self.processor.dispatch(input_state, message)?;
        use Transaction::*;

        let current_pos = e.absolute_pos();

        match e {
            Update((ref data, Phase::Drifting), v) => match self.state {
                State::Idle => {
                    self.state = State::Drifting {
                        start_t: message_time,
                        duration: Duration::from_secs(1),
                    };
                    e
                }
                State::Drifting { start_t, duration } => {
                    let dt = message_time - start_t;
                    let t = dt.as_secs_f64() / duration.as_secs_f64();
                    // note: target may be moving, so we ask for it each time.
                    let state = input_state.get_state::<S>().unwrap();
                    let target = (self.get_target)(state);
                    let f = (self.easing)(t);
                    let pt = current_pos.to_vector() * (1.0 - f) + target.to_vector() * f;
                    let v = v + (pt - current_pos.to_vector());
                    Update((data.clone(), Phase::Drifting), v)
                }
            },
            Commit((d, Phase::Drifting), v) => {
                let state = input_state.get_state::<S>().unwrap();
                let target = (self.get_target)(state);
                let v = v + (target - current_pos);
                Commit((d, Phase::Drifting), v)
            }
            e => {
                self.state = State::Idle;
                e
            }
        }
        .into()
    }
}
