use crate::{GestureRecognizer, InputState};
use emergent_drawing::{Point, Vector};
use emergent_ui::{WindowEvent, WindowMessage};

pub struct PanRecognizer {
    state: State,
}

#[derive(Clone, PartialEq, Debug)]
enum State {
    Waiting,
    Pressed(Point),
    Moved(Point, Vector),
}

#[derive(Clone, PartialEq, Debug)]
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

    fn dispatch(&mut self, _: &mut InputState, msg: WindowMessage) -> Option<Self::Event> {
        let (state, event) = match (self.state.clone(), msg.event) {
            (State::Waiting, event) if event.left_button_pressed() => {
                let position = msg.state.cursor_position().unwrap();
                (State::Pressed(position), Some(Event::Pressed(position)))
            }
            (State::Pressed(p), WindowEvent::CursorMoved(current)) => (
                State::Moved(p, current - p),
                Some(Event::Moved(p, current - p)),
            ),
            (State::Moved(p, _), WindowEvent::CursorMoved(current)) => (
                State::Moved(p, current - p),
                Some(Event::Moved(p, current - p)),
            ),
            (State::Pressed(p), event) if event.left_button_released() => {
                (State::Waiting, Some(Event::Released(p, Vector::default())))
            }
            (State::Moved(p, v), event) if event.left_button_released() => {
                (State::Waiting, Some(Event::Released(p, v)))
            }
            (state, _) => (state, None),
        };
        self.state = state;
        event
    }
}
