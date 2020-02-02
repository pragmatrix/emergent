//! State based animation.

use super::subscriptions::Subscription;
use crate::{GestureRecognizer, InputState};
use emergent_drawing::scalar;
use emergent_ui::{WindowEvent, WindowMessage};
use std::marker::PhantomData;
use std::time::{Duration, Instant};

pub trait StateAnimation<M> {
    fn active(&self) -> bool;
    fn animate(&mut self, v: scalar);
    fn commit(&mut self) {
        self.animate(1.0)
    }
}

struct Animator<S, M> {
    // TODO: animations must e simulatable.
    start_time: Instant,
    duration: Duration,
    easing: fn(scalar) -> scalar,
    pd: PhantomData<(*const S, *const M)>,
}

impl<S, M> Animator<S, M> {
    pub fn new(duration: Duration, easing: fn(scalar) -> scalar) -> Self
    where
        S: StateAnimation<M>,
    {
        const EASING_TOLERANCE: f64 = std::f64::EPSILON * 4.0;
        debug_assert!((easing(1.0) - 1.0).abs() <= EASING_TOLERANCE);
        debug_assert!(easing(0.0).abs() <= EASING_TOLERANCE);

        Self {
            start_time: Instant::now(),
            duration,
            easing,
            pd: PhantomData,
        }
    }
}

impl<S, M> GestureRecognizer for Animator<S, M>
where
    S: StateAnimation<M> + 'static,
{
    type Event = ();

    fn dispatch(
        &mut self,
        context: &mut InputState,
        message: WindowMessage,
    ) -> Option<Self::Event> {
        let state: &mut S = match context.get_mut() {
            None => {
                warn!("state gone, but animator survived (this could be an internal error)");
                return None;
            }
            Some(state) => state,
        };

        match message.event {
            WindowEvent::Tick(i) => {
                if i < self.start_time {
                    warn!("time skew, committing animation");
                    state.commit();
                    context.unsubscribe(Subscription::Ticks);
                    return None;
                }

                if i > self.start_time + self.duration {
                    state.commit();
                    context.unsubscribe(Subscription::Ticks);
                    return None;
                }

                let d = (i - self.start_time).as_secs_f64() / self.duration.as_secs_f64();
                state.animate((self.easing)(d));
                None
            }
            _ => None,
        }
    }
}

// https://gist.github.com/gre/1650294

pub mod easing {
    use emergent_drawing::scalar;

    // no easing, no acceleration
    pub fn linear(t: scalar) -> scalar {
        t
    }
    // accelerating from zero velocity
    pub fn ease_in_quad(t: scalar) -> scalar {
        t * t
    }
    // decelerating to zero velocity
    pub fn ease_out_quad(t: scalar) -> scalar {
        t * (2.0 - t)
    }
    // acceleration until halfway, then deceleration
    pub fn ease_in_out_quad(t: scalar) -> scalar {
        if t < 0.5 {
            2.0 * t * t
        } else {
            -1.0 + (4.0 - 2.0 * t) * t
        }
    }
    // accelerating from zero velocity
    pub fn ease_in_cubic(t: scalar) -> scalar {
        t * t * t
    }
    // decelerating to zero velocity
    pub fn ease_out_cubic(t: scalar) -> scalar {
        let t = t - 1.0;
        t * t * t + 1.0
    }
    // acceleration until halfway, then deceleration
    pub fn ease_in_out_cubic(t: scalar) -> scalar {
        if t < 0.5 {
            4.0 * t * t * t
        } else {
            (t - 1.0) * (2.0 * t - 2.0) * (2.0 * t - 2.0) + 1.0
        }
    }
    // accelerating from zero velocity
    pub fn ease_in_quart(t: scalar) -> scalar {
        t * t * t * t
    }
    // decelerating to zero velocity
    pub fn ease_out_quart(t: scalar) -> scalar {
        let t = t - 1.0;
        1.0 - t * t * t * t
    }
    // acceleration until halfway, then deceleration
    pub fn ease_in_out_quart(t: scalar) -> scalar {
        if t < 0.5 {
            8.0 * t * t * t * t
        } else {
            let t = t - 1.0;
            1.0 - 8.0 * t * t * t * t
        }
    }
    // accelerating from zero velocity
    pub fn ease_in_quint(t: scalar) -> scalar {
        t * t * t * t * t
    }
    // decelerating to zero velocity
    pub fn ease_out_quint(t: scalar) -> scalar {
        let t = t - 1.0;
        1.0 + t * t * t * t * t
    }
    // acceleration until halfway, then deceleration
    pub fn ease_in_out_quint(t: scalar) -> scalar {
        if t < 0.5 {
            16.0 * t * t * t * t * t
        } else {
            let t = t - 1.0;
            1.0 + 16.0 * t * t * t * t * t
        }
    }
}
