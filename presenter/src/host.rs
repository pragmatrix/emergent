use crate::{GestureRecognizer, Presenter, Support};
use emergent_drawing::Point;
use emergent_presentation::{Presentation, Scope};
use emergent_ui::{FrameLayout, WindowMsg};
use std::collections::HashMap;
use std::mem;
use std::rc::Rc;

pub struct Host<Msg> {
    pub support: Rc<Support>,
    /// A copy of the most recent presentation.
    /// This is primarily used for hit testing.
    pub presentation: Presentation,

    /// The active recognizers of the previous presentation.
    pub(crate) recognizers: HashMap<Vec<Scope>, Box<dyn GestureRecognizer<Msg = Msg>>>,
}

impl<Msg> Host<Msg> {
    pub fn new(support: Support) -> Host<Msg> {
        Host {
            support: Rc::new(support),
            presentation: Presentation::Empty,
            recognizers: HashMap::new(),
        }
    }

    pub fn present(
        &mut self,
        boundary: FrameLayout,
        present: impl FnOnce(&mut Presenter<Msg>),
    ) -> &Presentation {
        let active_recognizers = mem::replace(&mut self.recognizers, HashMap::new());
        let mut presenter = Presenter::new(self.support.clone(), boundary, active_recognizers);
        present(&mut presenter);
        // commit
        self.presentation = presenter.presentation;
        self.recognizers = presenter.recognizers;
        &self.presentation
    }

    /// Dispatches mouse input to a gesture recognizer and return a Msg if it produces one.
    pub fn dispatch_mouse_input(
        &mut self,
        (scope_path, _point): (Vec<Scope>, Point),
        msg: WindowMsg,
    ) -> Option<Msg>
    where
        Msg: 'static,
    {
        debug!("Hit scoped: {:?}", scope_path);

        self.recognizers
            .get_mut(&scope_path)
            .and_then(|recognizer| recognizer.update(msg))
    }
}
