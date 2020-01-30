use crate::recognizer::{Recognizer, RecognizerWithSubscription, Subscription};
use crate::{recognizer, ContextPath, ContextScope, GestureRecognizer, InputState, ScopedState};
use emergent_presentation::{PresentationPath, PresentationScope, Scoped};
use emergent_ui::WindowMessage;
use std::any::Any;
use std::collections::hash_map::RandomState;
use std::collections::HashSet;

type RecognizerResolver<Event> =
    Box<dyn Fn(&mut Box<dyn Any>) -> &mut dyn recognizer::Recognizer<Event>>;

pub(crate) struct RecognizerRecord<Event> {
    // used to map areas to the recognizer.
    presentation_path: PresentationPath,
    // used to know where the recognizer was created,
    context_path: ContextPath,
    // The recognizer needs to be stored as Any, because we want to recycle it later. If
    // we would store it as a GestureRecognizer trait, we could never resolve the
    // original type and can't use it as a state record.
    recognizer: Box<dyn Any>,
    // A function that converts the Box<Any> to a GestureRecognizer reference.
    resolver: RecognizerResolver<Event>,
}

impl<Event> RecognizerRecord<Event> {
    pub(crate) fn new<R>(recognizer: RecognizerWithSubscription<R>) -> Self
    where
        R: GestureRecognizer<Event = Event> + 'static,
    {
        let resolver: RecognizerResolver<Event> = Box::new(|r: &mut Box<dyn Any>| {
            r.downcast_mut::<RecognizerWithSubscription<R>>().unwrap()
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

impl<Event> GestureRecognizer for RecognizerRecord<Event> {
    type Event = Event;
    fn dispatch(
        &mut self,
        context: &mut InputState,
        message: WindowMessage,
    ) -> Option<Self::Event> {
        let recognizer = &mut self.recognizer;
        let resolver = &self.resolver;

        let recognizer = resolver(recognizer);
        recognizer.dispatch(context, message)
    }
}

impl<Event> Recognizer<Event> for RecognizerRecord<Event> {
    fn subscriptions(&mut self) -> &mut HashSet<Subscription, RandomState> {
        let recognizer = &mut self.recognizer;
        let resolver = &self.resolver;

        let recognizer = resolver(recognizer);
        recognizer.subscriptions()
    }
}
