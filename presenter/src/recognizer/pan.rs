use crate::recognizer::transaction::Transaction;
use crate::{InputProcessor, InputState};
use emergent_drawing::{Point, Vector};
use emergent_ui::{WindowEvent, WindowMessage};

pub struct Pan {
    state: State,
}

#[derive(Clone, PartialEq, Debug)]
enum State {
    Waiting,
    Pressed(Point),
    Moved(Point, Vector),
}

pub type Event = Transaction<Point>;

impl Default for Pan {
    fn default() -> Self {
        Self::new()
    }
}

impl Pan {
    pub fn new() -> Self {
        Self {
            state: State::Waiting,
        }
    }
}

impl InputProcessor for Pan {
    type In = WindowMessage;
    type Out = Event;

    fn dispatch(&mut self, _: &mut InputState, msg: WindowMessage) -> Option<Self::Out> {
        let (state, event) = match (self.state.clone(), msg.event) {
            (State::Waiting, event) if event.left_button_pressed() => {
                let position = msg.state.cursor_position().unwrap();
                (State::Pressed(position), Some(Event::Begin(position)))
            }
            (State::Pressed(p), WindowEvent::CursorMoved(current)) => (
                State::Moved(p, current - p),
                Some(Event::Update(p, current - p)),
            ),
            (State::Moved(p, _), WindowEvent::CursorMoved(current)) => (
                State::Moved(p, current - p),
                Some(Event::Update(p, current - p)),
            ),
            (State::Pressed(p), event) if event.left_button_released() => {
                (State::Waiting, Some(Event::Commit(p, Vector::ZERO)))
            }
            (State::Moved(p, v), event) if event.left_button_released() => {
                (State::Waiting, Some(Event::Commit(p, v)))
            }
            (state, _) => (state, None),
        };
        self.state = state;
        event
    }
}
