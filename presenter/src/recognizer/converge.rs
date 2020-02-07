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

use crate::recognizer::momentum;
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

impl<R> ConvergeTo for momentum::PreserveMomentum<R> {}

impl<P, TF, S> InputProcessor for Converge<P, TF, S>
where
    P: InputProcessor<In = WindowMessage, Out = momentum::Event> + Sized,
    TF: Fn(&S) -> Point,
    S: 'static,
{
    type In = WindowMessage;
    type Out = momentum::Event;

    fn dispatch(&mut self, input_state: &mut InputState, message: Self::In) -> Option<Self::Out> {
        let message_time = message.time;
        let e = self.processor.dispatch(input_state, message)?;

        match e {
            momentum::Event::Moved(p, v, momentum::Phase::Drifting) => match self.state {
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
                    // note: target may be moving, so we query it each time.
                    let state = input_state.get_state::<S>().unwrap();
                    let current = p + v;
                    let target = (self.get_target)(state);
                    let f = (self.easing)(t);
                    let pt = current.to_vector() * (1.0 - f) + target.to_vector() * f;
                    let v = pt - p.to_vector();
                    momentum::Event::Moved(p, v, momentum::Phase::Drifting)
                }
            },
            momentum::Event::End(p, _, momentum::Phase::Drifting) => {
                let state = input_state.get_state::<S>().unwrap();
                let target = (self.get_target)(state);
                let v = target - p;
                momentum::Event::End(p, v, momentum::Phase::Drifting)
            }
            e => {
                self.state = State::Idle;
                e
            }
        }
        .into()
    }
}
