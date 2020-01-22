use crate::{Context, PresentationPath, ScopedState, Support, View};
use emergent_drawing::Point;
use emergent_presentation::Presentation;
use emergent_ui::{FrameLayout, WindowMessage};
use std::mem;
use std::ops::Deref;
use std::rc::Rc;

pub struct Host<Msg> {
    support: Rc<Support>,

    /// The state of the presentation, scoped by the call hierarchy.
    state: ScopedState,

    /// The current view, containing the presentation and all the recognizers.
    view: View<Msg>,
}

impl<Msg> Host<Msg> {
    pub fn new(support: Support) -> Host<Msg> {
        Host {
            support: Rc::new(support),
            state: ScopedState::new(),
            view: View::new(),
        }
    }

    pub fn present(
        &mut self,
        boundary: FrameLayout,
        present: impl FnOnce(&mut Context) -> View<Msg>,
    ) {
        let state = mem::replace(&mut self.state, ScopedState::new());
        let mut context = Context::new(self.support.clone(), boundary, state);
        self.view = present(&mut context);
    }

    pub fn support(&self) -> &Support {
        self.support.deref()
    }

    pub fn presentation(&self) -> &Presentation {
        self.view.presentation()
    }

    /// Dispatches mouse input to a gesture recognizer and return a Msg if it produces one.
    pub fn dispatch_mouse_input(
        &mut self,
        (view_path, _point): (PresentationPath, Point),
        msg: WindowMessage,
    ) -> Option<Msg>
    where
        Msg: 'static,
    {
        debug!("Hit scoped: {:?}", view_path);

        // TODO: what about multiple hits?

        self.view.recognizer(&view_path).and_then(|r| r.update(msg))
    }
}
