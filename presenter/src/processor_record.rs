use crate::input_processor::Subscriptions;
use crate::processor::{Processor, ProcessorWithSubscription};
use crate::{processor, ContextPath, ContextScope, InputProcessor, InputState, ScopedState};
use emergent_presentation::{PresentationPath, PresentationScope, Scoped};
use emergent_ui::WindowMessage;
use std::any::Any;

type RecognizerResolver<Out> = Box<dyn Fn(&mut Box<dyn Any>) -> &mut dyn processor::Processor<Out>>;

pub(crate) struct ProcessorRecord<Event> {
    // used to map areas to the processor.
    presentation_path: PresentationPath,
    // used to know where the recognizer was created,
    context_path: ContextPath,
    // The recognizer needs to be stored as Any, because we want to recycle it later. If
    // we would store it as a InputProcessor trait, we could never resolve the
    // original type and can't use it as a state record.
    pub(crate) recognizer: Box<dyn Any>,
    // A function that converts the Box<Any> to a InputProcessor reference.
    resolver: RecognizerResolver<Event>,
}

impl<Event> ProcessorRecord<Event> {
    pub(crate) fn new<R>(recognizer: ProcessorWithSubscription<R>) -> Self
    where
        R: InputProcessor<In = WindowMessage, Out = Event> + 'static,
    {
        let resolver: RecognizerResolver<Event> = Box::new(|r: &mut Box<dyn Any>| {
            r.downcast_mut::<ProcessorWithSubscription<R>>().unwrap()
        });

        Self {
            presentation_path: Default::default(),
            context_path: Default::default(),
            recognizer: Box::new(recognizer),
            resolver,
        }
    }

    pub fn presentation_path(&self) -> &PresentationPath {
        &self.presentation_path
    }

    pub fn context_path(&self) -> &ContextPath {
        &self.context_path
    }

    pub fn context_scoped(self, scope: impl Into<ContextScope>) -> Self {
        let scope = scope.into();
        Self {
            context_path: self.context_path.scoped(scope),
            ..self
        }
    }

    pub fn presentation_scoped(self, scope: impl Into<PresentationScope>) -> Self {
        let scope = scope.into();
        Self {
            presentation_path: self.presentation_path.scoped(scope),
            ..self
        }
    }

    pub fn into_scoped_state(self) -> ScopedState {
        (self.context_path, self.recognizer)
    }
}

impl<Event> InputProcessor for ProcessorRecord<Event> {
    type In = WindowMessage;
    type Out = Event;
    fn dispatch(&mut self, context: &mut InputState, message: WindowMessage) -> Option<Event> {
        let recognizer = &mut self.recognizer;
        let resolver = &self.resolver;

        let recognizer = resolver(recognizer);
        recognizer.dispatch(context, message)
    }
}

impl<Event> Processor<Event> for ProcessorRecord<Event> {
    fn subscriptions(&mut self) -> &mut Subscriptions {
        let recognizer = &mut self.recognizer;
        let resolver = &self.resolver;

        let recognizer = resolver(recognizer);
        recognizer.subscriptions()
    }
}
