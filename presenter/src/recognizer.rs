use crate::{InputProcessor, InputState};
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

// Below follows a rather convoluted way of transporting a input processor including its subscription
// state through a `Box<Any>`.
// TODO: find a simpler way.

pub(crate) trait Recognizer<Out>: InputProcessor<In = WindowMessage, Out = Out> {
    fn subscriptions(&mut self) -> &mut Subscriptions;
}

pub(crate) struct RecognizerWithSubscription<R>
where
    R: InputProcessor,
{
    pub recognizer: R,
    pub subscriptions: Subscriptions,
}

impl<R> From<R> for RecognizerWithSubscription<R>
where
    R: InputProcessor,
{
    fn from(r: R) -> Self {
        Self {
            recognizer: r,
            subscriptions: Subscriptions::default(),
        }
    }
}

impl<R> Recognizer<R::Out> for RecognizerWithSubscription<R>
where
    R: InputProcessor<In = WindowMessage>,
{
    fn subscriptions(&mut self) -> &mut Subscriptions {
        &mut self.subscriptions
    }
}

impl<R> InputProcessor for RecognizerWithSubscription<R>
where
    R: InputProcessor,
{
    type In = R::In;
    type Out = R::Out;

    fn dispatch(&mut self, context: &mut InputState, message: Self::In) -> Option<Self::Out> {
        self.recognizer.dispatch(context, message)
    }
}
