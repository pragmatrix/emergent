use crate::input_processor::Subscriptions;
use crate::processor::{Processor, ProcessorWithSubscription};
use crate::{processor, ContextPath, ContextScope, InputProcessor, InputState, ScopedState};
use emergent_presentation::{PresentationPath, PresentationScope, Scoped};
use emergent_ui::WindowMessage;
use std::any::Any;

type ProcessorResolver<Out> = Box<dyn Fn(&mut Box<dyn Any>) -> &mut dyn processor::Processor<Out>>;

pub(crate) struct ProcessorRecord<Event> {
    // used to map areas to the processor.
    presentation_path: PresentationPath,
    // used to know where the processor was created,
    context_path: ContextPath,
    // The processor needs to be stored as Any, because we want to recycle it later. If
    // we would store it as a InputProcessor trait, we could never resolve the
    // original type and can't use it as a state record.
    pub(crate) processor: Box<dyn Any>,
    // A function that converts the Box<Any> to a InputProcessor reference.
    resolver: ProcessorResolver<Event>,
}

impl<Event> ProcessorRecord<Event> {
    pub(crate) fn new<R>(processor: ProcessorWithSubscription<R>) -> Self
    where
        R: InputProcessor<In = WindowMessage, Out = Event> + 'static,
    {
        let resolver: ProcessorResolver<Event> = Box::new(|r: &mut Box<dyn Any>| {
            r.downcast_mut::<ProcessorWithSubscription<R>>().unwrap()
        });

        Self {
            presentation_path: Default::default(),
            context_path: Default::default(),
            processor: Box::new(processor),
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
        (self.context_path, self.processor)
    }
}

impl<Event> InputProcessor for ProcessorRecord<Event> {
    type In = WindowMessage;
    type Out = Event;
    fn dispatch(&mut self, context: &mut InputState, message: WindowMessage) -> Option<Event> {
        let processor = &mut self.processor;
        let resolver = &self.resolver;

        let processor = resolver(processor);
        processor.dispatch(context, message)
    }
}

impl<Event> Processor<Event> for ProcessorRecord<Event> {
    fn subscriptions(&mut self) -> &mut Subscriptions {
        let processor = &mut self.processor;
        let resolver = &self.resolver;

        let processor = resolver(processor);
        processor.subscriptions()
    }
}
