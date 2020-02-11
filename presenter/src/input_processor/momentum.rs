//! This input processor adds a drifting phase after a regular Pan processor by subscribing to ticks for a
//! pre-specified duration.
//!
//! Notes:
//!
//! - 0.25 seems to be a good smoothing value for the velocity tracker.
//!   The less it is, the harder it is for the user to initiate a drifting phase.
//! - The duration is needed to account for the tick subscriptions. Previously I wanted to do this separately, but I saw
//!   no way to get the subscriptions consistent.

use crate::input_processor::transaction::Transaction;
use crate::input_processor::{Subscriber, Subscription, Subscriptions};
use crate::{velocity, InputProcessor, InputState};
use emergent_drawing::{scalar, Point, Vector};
use emergent_ui::{WindowEvent, WindowMessage};
use std::fmt::Debug;
use std::time::{Duration, Instant};

impl<T> PreserveMomentum for T where T: InputProcessor {}

pub trait PreserveMomentum: Sized {
    fn preserve_momentum(
        self,
        velocity_threshold: scalar,
        drift_easing: fn(scalar) -> scalar,
        drift_duration: Duration,
    ) -> Momentum<Self>
    where
        Self: InputProcessor<In = WindowMessage, Out = Transaction<Point>>,
    {
        Momentum {
            processor: self,
            velocity_threshold,
            drift_easing,
            drift_duration,
            state: State::Idle,
        }
    }
}

#[derive(Debug)]
enum State {
    Idle,
    Interacting(velocity::Tracker),
    Drifting {
        start_p: Point,
        start_time: Instant,
        drift_way_v: Vector,
    },
}

#[derive(Debug)]
pub struct Momentum<P> {
    processor: P,
    velocity_threshold: scalar,
    drift_easing: fn(scalar) -> scalar,
    drift_duration: Duration,
    state: State,
}

#[derive(Copy, Clone, Debug)]
pub enum Phase {
    Interacting,
    Drifting,
}

impl<P> InputProcessor for Momentum<P>
where
    P: InputProcessor<In = WindowMessage, Out = Transaction<Point>>,
{
    type In = WindowMessage;
    type Out = Transaction<(Point, Phase)>;

    fn dispatch(&mut self, input_state: &mut InputState, message: Self::In) -> Option<Self::Out> {
        let e = self.processor.dispatch(input_state, message.clone());
        use Transaction::*;

        match &mut self.state {
            State::Idle => match e {
                Some(Begin(p)) => Some(self.begin(message, p)),
                e => {
                    warn!("unprocessed event: {:?}", e);
                    None
                }
            },
            State::Interacting(tracker) => match e {
                Some(Update(p)) => {
                    tracker.measure(message.time, p);
                    Some(Update((p, Phase::Interacting)))
                }
                Some(Commit(p)) => {
                    // even though v is most likely at the previous Event::Moved coordinate, it is important to
                    // once more send this to the tracker, because of the updated timestamp.
                    let velocity = tracker.measure(message.time, p);

                    if velocity.length() < self.velocity_threshold {
                        info!(
                            "velocity {:?} too low to reach velocity threshold of {:?}, ending",
                            velocity, self.velocity_threshold
                        );
                        self.state = State::Idle;
                        Some(Commit((p, Phase::Interacting)))
                    } else {
                        self.state = State::Drifting {
                            start_p: p,
                            start_time: message.time,
                            drift_way_v: velocity * self.drift_duration.as_secs_f64(),
                        };
                        Some(Update((p, Phase::Interacting)))
                    }
                }
                e => {
                    warn!("unprocessed event: {:?}", e);
                    None
                }
            },
            State::Drifting {
                start_p,
                start_time,
                drift_way_v: drift_way,
            } => match (e, &message.event) {
                (Some(Begin(p)), _) => Some(self.begin(message, p)),
                (None, WindowEvent::Tick(t2)) => {
                    // TODO: handle time drift here?
                    let dt = *t2 - *start_time;
                    if dt < self.drift_duration {
                        let t = dt.as_secs_f64() / self.drift_duration.as_secs_f64();
                        let p = *start_p + *drift_way * (self.drift_easing)(t);
                        Some(Update((p, Phase::Drifting)))
                    } else {
                        let p = *start_p + *drift_way;
                        self.state = State::Idle;
                        Some(Commit((p, Phase::Drifting)))
                    }
                }
                e => {
                    warn!("unprocessed event: {:?}", e);
                    None
                }
            },
        }
    }
}

impl<P> Subscriber for Momentum<P>
where
    P: Subscriber,
{
    fn subscriptions(&self) -> Subscriptions {
        let mut subs = self.processor.subscriptions();
        match self.state {
            State::Idle => {}
            State::Interacting(_) => {}
            State::Drifting { .. } => subs.subscribe(Subscription::Ticks),
        }
        subs
    }
}

impl<P> Momentum<P> {
    fn begin(&mut self, message: WindowMessage, p: Point) -> Transaction<(Point, Phase)> {
        let mut tracker = velocity::Tracker::new(0.25);
        tracker.measure(message.time, p);
        self.state = State::Interacting(tracker);
        Transaction::Begin((p, Phase::Interacting))
    }
}
