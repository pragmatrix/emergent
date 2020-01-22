use crate::GestureRecognizer;
use emergent_drawing::{Point, Vector};
use emergent_ui::{WindowEvent, WindowMessage};

pub struct PanRecognizer {
    state: State,
}

#[derive(Clone, Debug)]
enum State {
    Waiting,
    Pressed(Point),
    Moved(Point, Vector),
}

pub enum Event {
    Pressed(Point),
    Moved(Point, Vector),
    Released(Point, Vector),
}

impl PanRecognizer {
    pub fn new() -> Self {
        Self {
            state: State::Waiting,
        }
    }
}

impl GestureRecognizer for PanRecognizer {
    type Event = Event;

    fn update(&mut self, msg: WindowMessage) -> Option<Self::Event> {
        let (state, event) = match (self.state.clone(), msg.event) {
            (State::Waiting, event) if event.left_button_pressed() => {
                let position = msg.state.cursor_position().unwrap();
                (State::Pressed(position), Some(Event::Pressed(position)))
            }
            (State::Pressed(p), WindowEvent::CursorMoved(current)) => (
                State::Moved(p, current - p),
                Some(Event::Moved(p, current - p)),
            ),
            (state, _) => (state, None),
        };
        self.state = state;
        event
    }
}
