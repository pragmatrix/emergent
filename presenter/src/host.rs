use crate::{GestureRecognizer, Presenter, Support};
use emergent_drawing::{Point, ReplaceWith};
use emergent_presentation::{Presentation, Scope};
use emergent_ui::{FrameLayout, ModifiersState, MouseButton, WindowMsg};
use std::collections::HashMap;

pub struct Host<Msg> {
    pub support: Support,
    /// A copy of the most recent presentation.
    /// This is primarily used for hit testing.
    pub presentation: Presentation,

    /// The active recognizers of the previous presentation.
    pub(crate) recognizers: HashMap<Vec<Scope>, Box<dyn GestureRecognizer<Msg = Msg>>>,
}

impl<Msg: 'static> Host<Msg> {
    pub fn new(support: Support) -> Host<Msg> {
        Host {
            support,
            presentation: Presentation::Empty,
            recognizers: HashMap::new(),
        }
    }

    pub fn present(
        &mut self,
        boundary: FrameLayout,
        present: impl FnOnce(&mut Presenter<Msg>),
    ) -> &Presentation {
        self.replace_with(|h| {
            let mut presenter = Presenter::new(h, boundary);
            present(&mut presenter);
            // commit
            let mut host = presenter.host;
            host.presentation = presenter.presentation;
            host.recognizers = presenter.recognizers;
            host
        });
        &self.presentation
    }

    /// Dispatches mouse input to a gesture recognizer and return a Msg if it produces one.
    pub fn dispatch_mouse_input(
        &mut self,
        (scope_path, point): (Vec<Scope>, Point),
        msg: WindowMsg,
    ) -> Option<Msg> {
        debug!("Hit scoped: {:?}", scope_path);

        self.recognizers
            .get_mut(&scope_path)
            .and_then(|recognizer| recognizer.update(msg))
    }
}
