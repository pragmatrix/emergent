//! This input processor adds a drifting phase after a regular Pan recognizer by subscribing to ticks for a
//! pre-specified duration.
//!
//! Notes:
//!
//! - 0.25 seems to be a good smoothing value for the velocity tracker.
//!   The less it is, the harder it is for the user to initiate a drifting phase.
//! - The duration is needed to account for the tick subscriptions. Previously I wanted to do this separately, but I saw
//!   no way to get the subscriptions consistent.

use crate::recognizer::{pan, Pan, Subscription, Translate};
use crate::{velocity, InputProcessor, InputState};
use emergent_drawing::{scalar, Point, Vector};
use emergent_ui::{WindowEvent, WindowMessage};
use std::time::{Duration, Instant};

impl Pan {
    pub fn preserve_momentum(
        self,
        velocity_threshold: scalar,
        drift_easing: fn(scalar) -> scalar,
        drift_duration: Duration,
    ) -> PreserveMomentum<Self> {
        PreserveMomentum {
            recognizer: self,
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
        p: Point,
        start_v: Vector,
        start_time: Instant,
        drift_way_v: Vector,
    },
}

#[derive(Debug)]
pub struct PreserveMomentum<R> {
    recognizer: R,
    velocity_threshold: scalar,
    drift_easing: fn(scalar) -> scalar,
    drift_duration: Duration,
    state: State,
}

#[derive(Clone, Debug)]
pub enum Event {
    Begin(Point),
    Moved(Point, Vector, Phase),
    End(Point, Vector, Phase),
}

#[derive(Copy, Clone, Debug)]
pub enum Phase {
    Interacting,
    Drifting,
}

impl Event {
    pub fn phase(&self) -> Phase {
        match self {
            Event::Begin(_) => Phase::Interacting,
            Event::Moved(_, _, ph) | Event::End(_, _, ph) => *ph,
        }
    }
}

impl Translate for Event {
    fn translate(self, t: Vector) -> Self
    where
        Self: Sized,
    {
        match self {
            Event::Begin(_) => {
                error!("translating a Begin event does not make sense, the location where an interaction started \
                    can not be changed");
                self
            }
            Event::Moved(p, v, ph) => Event::Moved(p, v + t, ph),
            Event::End(p, v, ph) => Event::Moved(p, v + t, ph),
        }
    }
}

impl Into<pan::Event> for Event {
    fn into(self) -> pan::Event {
        match self {
            Event::Begin(p) => pan::Event::Begin(p),
            Event::Moved(p, v, _) => pan::Event::Moved(p, v),
            Event::End(p, v, _) => pan::Event::End(p, v),
        }
    }
}

impl<R> InputProcessor for PreserveMomentum<R>
where
    R: InputProcessor<In = WindowMessage, Out = pan::Event>,
{
    type In = WindowMessage;
    type Out = Event;

    fn dispatch(&mut self, input_state: &mut InputState, message: Self::In) -> Option<Self::Out> {
        info!("momemtum in: {:?}", message);
        let e = self.recognizer.dispatch(input_state, message.clone());
        info!("momemtum out: {:?}", e);

        match self.state {
            State::Idle => match e {
                Some(pan::Event::Begin(p)) => Some(self.begin(message, p)),
                e => {
                    warn!("unprocessed event: {:?}", e);
                    None
                }
            },
            State::Interacting(ref mut tracker) => match e {
                Some(pan::Event::Moved(p, v)) => {
                    tracker.measure(message.time, p + v);
                    Some(Event::Moved(p, v, Phase::Interacting))
                }
                Some(pan::Event::End(p, v)) => {
                    // even though v is most likely at the previous Event::Moved coordinate, it is important to
                    // once more send this to the tracker, because of the updated timestamp.
                    let velocity = tracker.measure(message.time, p + v);

                    if velocity.length() < self.velocity_threshold {
                        info!(
                            "velocity {:?} too low to reach velocity threshold of {:?}, ending",
                            velocity, self.velocity_threshold
                        );
                        self.state = State::Idle;
                        Some(Event::End(p, v, Phase::Interacting))
                    } else {
                        input_state.subscribe(Subscription::Ticks);
                        self.state = State::Drifting {
                            p,
                            start_v: v,
                            start_time: message.time,
                            drift_way_v: velocity * self.drift_duration.as_secs_f64(),
                        };
                        Some(Event::Moved(p, v, Phase::Interacting))
                    }
                }
                e => {
                    warn!("unprocessed event: {:?}", e);
                    None
                }
            },
            State::Drifting {
                p,
                start_v,
                start_time,
                drift_way_v: drift_way,
            } => match (e, &message.event) {
                (Some(pan::Event::Begin(p)), _) => {
                    input_state.unsubscribe(Subscription::Ticks);
                    Some(self.begin(message, p))
                }
                (None, WindowEvent::Tick(t2)) => {
                    // TODO: handle time drift here?
                    let dt = *t2 - start_time;
                    if dt < self.drift_duration {
                        let t = dt.as_secs_f64() / self.drift_duration.as_secs_f64();
                        let v = start_v + drift_way * (self.drift_easing)(t);
                        Some(Event::Moved(p, v, Phase::Drifting))
                    } else {
                        input_state.unsubscribe(Subscription::Ticks);
                        self.state = State::Idle;
                        let v = start_v + drift_way;
                        Some(Event::End(p, v, Phase::Drifting))
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

impl<R> PreserveMomentum<R> {
    fn begin(&mut self, message: WindowMessage, p: Point) -> Event {
        let mut tracker = velocity::Tracker::new(0.25);
        tracker.measure(message.time, p);
        self.state = State::Interacting(tracker);
        Event::Begin(p)
    }
}
