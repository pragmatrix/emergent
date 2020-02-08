use crate::input_processor::Subscriptions;
use crate::{InputProcessor, InputState};
use emergent_ui::WindowMessage;

// Below follows a rather convoluted way of transporting a input processor including its subscription
// state through a `Box<Any>`.
// TODO: find a simpler way.

pub(crate) trait Processor<Out>: InputProcessor<In = WindowMessage, Out = Out> {
    fn subscriptions(&mut self) -> &mut Subscriptions;
}

pub struct ProcessorWithSubscription<R>
where
    R: InputProcessor,
{
    pub recognizer: R,
    pub subscriptions: Subscriptions,
}

impl<R> From<R> for ProcessorWithSubscription<R>
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

impl<R> Processor<R::Out> for ProcessorWithSubscription<R>
where
    R: InputProcessor<In = WindowMessage>,
{
    fn subscriptions(&mut self) -> &mut Subscriptions {
        &mut self.subscriptions
    }
}

impl<R> InputProcessor for ProcessorWithSubscription<R>
where
    R: InputProcessor,
{
    type In = R::In;
    type Out = R::Out;

    fn dispatch(&mut self, context: &mut InputState, message: Self::In) -> Option<Self::Out> {
        self.recognizer.dispatch(context, message)
    }
}
