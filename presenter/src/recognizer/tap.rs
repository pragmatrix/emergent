use crate::{InputProcessor, InputState};
use emergent_drawing::Point;
use emergent_ui::WindowMessage;

pub struct TapRecognizer {}

pub enum Event {
    Tapped(Point),
}

impl Default for TapRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

impl TapRecognizer {
    pub fn new() -> Self {
        Self {}
    }
}

impl InputProcessor for TapRecognizer {
    type In = WindowMessage;
    type Out = Event;
    fn dispatch(&mut self, _: &mut InputState, msg: WindowMessage) -> Option<Event> {
        if msg.event.left_button_pressed() {
            let position = msg.state.cursor_position().unwrap();
            Some(Event::Tapped(position))
        } else {
            None
        }
    }
}
