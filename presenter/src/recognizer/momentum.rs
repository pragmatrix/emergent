//! This input processor adds a drifting phase after a regular Pan recognizer by subscribing to ticks for a
//! pre-specified duration.
//!
//! Notes:
//!
//! - 0.25 seems to be a good smoothing value for the velocity tracker.
//!   The less it is, the harder it is for the user to initiate a drifting phase.
//! - The duration is needed to account for the tick subscriptions. Previously I wanted to do this separately, but I saw
//!   no way to get the subscriptions consistent.

use crate::recognizer::transaction::{AbsolutePos, Transaction};
use crate::recognizer::Subscription;
use crate::{velocity, InputProcessor, InputState};
use emergent_drawing::{scalar, Point, Vector};
use emergent_ui::{WindowEvent, WindowMessage};
use std::fmt::Debug;
use std::time::{Duration, Instant};

impl<T> PreserveMomentum for T where T: InputProcessor {}

pub trait PreserveMomentum: Sized {
    fn preserve_momentum<Data>(
        self,
        velocity_threshold: scalar,
        drift_easing: fn(scalar) -> scalar,
        drift_duration: Duration,
    ) -> Momentum<Self, Data>
    where
        Self: InputProcessor<In = WindowMessage, Out = Transaction<Data>>,
    {
        Momentum {
            recognizer: self,
            velocity_threshold,
            drift_easing,
            drift_duration,
            state: State::Idle,
        }
    }
}

#[derive(Debug)]
enum State<Data> {
    Idle,
    Interacting(velocity::Tracker),
    Drifting {
        data: Data,
        p: Point,
        start_v: Vector,
        start_time: Instant,
        drift_way_v: Vector,
    },
}

#[derive(Debug)]
pub struct Momentum<R, Data> {
    recognizer: R,
    velocity_threshold: scalar,
    drift_easing: fn(scalar) -> scalar,
    drift_duration: Duration,
    state: State<Data>,
}

impl<Data> AbsolutePos for Transaction<(Data, Phase)>
where
    Transaction<Data>: AbsolutePos,
    Self: Clone,
    Data: Debug,
{
    fn absolute_pos(&self) -> Point {
        self.clone().map_data(|(d, p)| d).absolute_pos()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Phase {
    Interacting,
    Drifting,
}

impl<R, Data> InputProcessor for Momentum<R, Data>
where
    R: InputProcessor<In = WindowMessage, Out = Transaction<Data>>,
    Transaction<Data>: AbsolutePos,
    Data: Clone,
{
    type In = WindowMessage;
    type Out = Transaction<(Data, Phase)>;

    fn dispatch(&mut self, input_state: &mut InputState, message: Self::In) -> Option<Self::Out> {
        let e = self.recognizer.dispatch(input_state, message.clone());
        use Transaction::*;

        match &mut self.state {
            State::Idle => match e {
                Some(e @ Begin(_)) => Some(self.begin(message, e.data().clone(), e.absolute_pos())),
                e => {
                    warn!("unprocessed event: {:?}", e);
                    None
                }
            },
            State::Interacting(tracker) => match e {
                Some(e @ Update(_, _)) => {
                    tracker.measure(message.time, e.absolute_pos());
                    Some(e.map_data(|p| (p, Phase::Interacting)))
                }
                Some(e @ Commit(_, _)) => {
                    // even though v is most likely at the previous Event::Moved coordinate, it is important to
                    // once more send this to the tracker, because of the updated timestamp.
                    let velocity = tracker.measure(message.time, e.absolute_pos());

                    if velocity.length() < self.velocity_threshold {
                        info!(
                            "velocity {:?} too low to reach velocity threshold of {:?}, ending",
                            velocity, self.velocity_threshold
                        );
                        self.state = State::Idle;
                        Some(e.map_data(|p| (p, Phase::Interacting)))
                    } else {
                        input_state.subscribe(Subscription::Ticks);
                        let p = e.absolute_pos();
                        self.state = State::Drifting {
                            data: e.data().clone(),
                            p: p - e.v(),
                            start_v: e.v(),
                            start_time: message.time,
                            drift_way_v: velocity * self.drift_duration.as_secs_f64(),
                        };
                        Some(e.map_data(|p| (p, Phase::Interacting)))
                    }
                }
                e => {
                    warn!("unprocessed event: {:?}", e);
                    None
                }
            },
            State::Drifting {
                data,
                p,
                start_v,
                start_time,
                drift_way_v: drift_way,
            } => match (e, &message.event) {
                (Some(e @ Begin(_)), _) => {
                    let p = e.absolute_pos();
                    input_state.unsubscribe(Subscription::Ticks);
                    Some(self.begin(message, e.data().clone(), p))
                }
                (None, WindowEvent::Tick(t2)) => {
                    // TODO: handle time drift here?
                    let dt = *t2 - *start_time;
                    if dt < self.drift_duration {
                        let t = dt.as_secs_f64() / self.drift_duration.as_secs_f64();
                        let v = *start_v + *drift_way * (self.drift_easing)(t);
                        Some(Update((data.clone(), Phase::Drifting), v))
                    } else {
                        input_state.unsubscribe(Subscription::Ticks);
                        let v = *start_v + *drift_way;
                        let data = data.clone();
                        self.state = State::Idle;
                        Some(Commit((data, Phase::Drifting), v))
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

impl<R, Data> Momentum<R, Data> {
    fn begin(
        &mut self,
        message: WindowMessage,
        data: Data,
        p: Point,
    ) -> Transaction<(Data, Phase)> {
        let mut tracker = velocity::Tracker::new(0.25);
        tracker.measure(message.time, p);
        self.state = State::Interacting(tracker);
        Transaction::Begin((data, Phase::Interacting))
    }
}
