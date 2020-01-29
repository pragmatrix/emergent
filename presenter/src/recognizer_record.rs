use crate::{
    ContextPath, ContextScope, GestureRecognizer, InputState, PresentationPath, PresentationScope,
    ScopedState,
};
use emergent_presentation::Scoped;
use emergent_ui::WindowMessage;
use std::any::Any;

type RecognizerResolver<Msg> =
    Box<dyn Fn(&mut Box<dyn Any>) -> &mut dyn GestureRecognizer<Event = Msg>>;

pub struct RecognizerRecord<Msg> {
    // used to map areas to the recognizer.
    presentation_path: PresentationPath,
    // used to know to which context belongs created,
    context_path: ContextPath,
    // The recognizer needs to be stored as Any, because we want to recycle it later. If
    // we would store it as a GestureRecognizer trait, we could never resolve the
    // original type and can't use it as a state record.
    recognizer: Box<dyn Any>,
    // A function that converts the Box<Any> to a GestureRecognizer reference.
    resolver: RecognizerResolver<Msg>,
}

impl<Msg> RecognizerRecord<Msg> {
    pub fn new(recognizer: Box<dyn Any>, resolver: RecognizerResolver<Msg>) -> Self {
        Self {
            presentation_path: Default::default(),
            context_path: Default::default(),
            recognizer,
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

impl<Msg> GestureRecognizer for RecognizerRecord<Msg> {
    type Event = Msg;
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
