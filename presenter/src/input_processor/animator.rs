//! State based animation.

use super::subscriptions::Subscription;
use crate::interpolated::Interpolated;
use crate::{InputProcessor, InputState};
use emergent_drawing::scalar;
use emergent_ui::{WindowEvent, WindowMessage};
use std::time::{Duration, Instant};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Event {
    Animate(scalar),
    Commit,
}

impl Event {
    pub fn t(self) -> scalar {
        match self {
            Event::Animate(t) => t,
            Event::Commit => 1.0,
        }
    }

    pub fn interpolate<I>(self, from: &I, to: &I) -> I
    where
        I: Interpolated,
    {
        from.interpolated(to, self.t())
    }
}

pub struct Animator {
    // TODO: animations must be simulatable.
    start_time: Instant,
    duration: Duration,
    easing: fn(scalar) -> scalar,
}

impl Animator {
    pub fn new(duration: Duration, easing: fn(scalar) -> scalar) -> Self {
        const EASING_TOLERANCE: f64 = std::f64::EPSILON * 4.0;
        debug_assert!((easing(1.0) - 1.0).abs() <= EASING_TOLERANCE);
        debug_assert!(easing(0.0).abs() <= EASING_TOLERANCE);

        Self {
            start_time: Instant::now(),
            duration,
            easing,
        }
    }
}

impl InputProcessor for Animator {
    type In = WindowMessage;
    type Out = Event;

    fn dispatch(&mut self, context: &mut InputState, message: WindowMessage) -> Option<Self::Out> {
        match message.event {
            WindowEvent::Tick(i) => {
                if i < self.start_time {
                    warn!("time skew, committing animation");
                    context.unsubscribe(Subscription::Ticks);
                    return Some(Event::Commit);
                }

                if i > self.start_time + self.duration {
                    context.unsubscribe(Subscription::Ticks);
                    return Some(Event::Commit);
                }

                let d = (i - self.start_time).as_secs_f64() / self.duration.as_secs_f64();
                Some(Event::Animate((self.easing)(d)))
            }
            _ => None,
        }
    }
}
