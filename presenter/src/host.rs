use crate::{GestureRecognizer, Presenter, Support};
use emergent_drawing::Point;
use emergent_presentation::{Presentation, Scope};
use emergent_ui::{FrameLayout, WindowMsg};
use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;
use std::{any, mem};

pub type ComponentKey = (Vec<Scope>, any::TypeId);
pub type ComponentPool = HashMap<ComponentKey, Box<dyn Any>>;

pub struct Host<Msg> {
    pub support: Rc<Support>,
    /// A copy of the most recent presentation.
    /// This is primarily used for hit testing.
    pub presentation: Presentation,

    /// The active components of the previous presentation.
    pub(crate) components: ComponentPool,

    /// The active recognizers of the previous presentation.
    pub(crate) recognizers: HashMap<Vec<Scope>, Box<dyn GestureRecognizer<Msg = Msg>>>,
}

impl<Msg> Host<Msg> {
    pub fn new(support: Support) -> Host<Msg> {
        Host {
            support: Rc::new(support),
            presentation: Presentation::Empty,
            components: ComponentPool::new(),
            recognizers: HashMap::new(),
        }
    }

    pub fn present(
        &mut self,
        boundary: FrameLayout,
        present: impl FnOnce(&mut Presenter<Msg>),
    ) -> &Presentation {
        let active_recognizers = mem::replace(&mut self.recognizers, HashMap::new());
        let active_components = mem::replace(&mut self.components, ComponentPool::new());
        let mut presenter = Presenter::new(
            self.support.clone(),
            boundary,
            active_recognizers,
            active_components,
        );
        present(&mut presenter);
        // commit
        self.presentation = presenter.presentation;
        self.recognizers = presenter.recognizers;
        self.components = presenter.components;
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
