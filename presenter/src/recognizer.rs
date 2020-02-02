use crate::{GestureRecognizer, InputState};
use emergent_ui::WindowMessage;

mod animator;
pub use animator::*;

pub mod mover;
pub use mover::MoverRecognizer;

pub mod pan;
pub use pan::PanRecognizer;

mod subscriptions;
pub use subscriptions::*;

pub mod tap;
pub use tap::TapRecognizer;

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
