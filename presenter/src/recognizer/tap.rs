use crate::GestureRecognizer;
use emergent_drawing::Point;
use emergent_ui::{ElementState, MouseButton, WindowMsg};

pub struct TapRecognizer<Msg> {
    tapped: Box<dyn Fn() -> Msg>,
}

impl<Msg: 'static> TapRecognizer<Msg> {
    pub fn new(tapped: impl Fn() -> Msg + 'static) -> Self {
        Self {
            tapped: Box::new(tapped),
        }
    }
}

impl<Msg: 'static> GestureRecognizer for TapRecognizer<Msg> {
    type Msg = Msg;
    fn update(&mut self, msg: WindowMsg) -> Option<Msg> {
        match msg {
            WindowMsg::MouseInput { state, button, .. }
                if state == ElementState::Pressed && button == MouseButton::Left =>
            {
                Some((self.tapped)())
            }
            _ => None,
        }
    }
}
