use crate::{GestureRecognizer, InputState};
use emergent_ui::{MouseButton, WindowMessage};
pub use mover::MoverRecognizer;
use std::collections::HashSet;

pub mod pan;
pub use pan::PanRecognizer;

pub mod tap;
pub use tap::TapRecognizer;

pub mod mover;

pub type Subscriptions = HashSet<Subscription>;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Subscription {
    /// Subscribed to continuity events after a specific mouse button was pressed.
    ///
    /// These are automatically subscribed for every recognizer that receives button presses.
    ButtonContinuity(MouseButton),
    /// Subscribed to timer ticks once per frame.
    Ticks,
}

// Below follows a rather convoluted way of transporting a gesture recognizer including its subscription
// state through a `Box<Any>`.
// TODO: find a simpler way.

pub(crate) trait Recognizer<Event>: GestureRecognizer<Event = Event> {
    fn subscriptions(&mut self) -> &mut Subscriptions;
}

pub(crate) struct RecognizerWithSubscription<R>
where
    R: GestureRecognizer,
{
    pub recognizer: R,
    pub subscriptions: Subscriptions,
}

impl<R> From<R> for RecognizerWithSubscription<R>
where
    R: GestureRecognizer,
{
    fn from(r: R) -> Self {
        Self {
            recognizer: r,
            subscriptions: Subscriptions::default(),
        }
    }
}

impl<R> Recognizer<R::Event> for RecognizerWithSubscription<R>
where
    R: GestureRecognizer,
{
    fn subscriptions(&mut self) -> &mut Subscriptions {
        &mut self.subscriptions
    }
}

impl<R> GestureRecognizer for RecognizerWithSubscription<R>
where
    R: GestureRecognizer,
{
    type Event = R::Event;

    fn dispatch(
        &mut self,
        context: &mut InputState,
        message: WindowMessage,
    ) -> Option<Self::Event> {
        self.recognizer.dispatch(context, message)
    }
}
