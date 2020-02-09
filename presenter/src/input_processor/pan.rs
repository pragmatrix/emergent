use crate::input_processor::move_threshold::WithMoveThreshold;
use crate::input_processor::transaction::Transaction;
use crate::{InputProcessor, InputState};
use emergent_drawing::Point;
use emergent_ui::{WindowEvent, WindowMessage};

pub enum Pan {}

impl Pan {
    pub fn new() -> impl InputProcessor<In = WindowMessage, Out = Transaction<Point>> {
        PanProcessor {
            state: State::Waiting,
        }
        .with_move_threshold(10.0)
    }
}

pub type Event = Transaction<Point>;

struct PanProcessor {
    state: State,
}

#[derive(Clone, PartialEq, Debug)]
enum State {
    Waiting,
    Pressed,
    Moved,
}

impl InputProcessor for PanProcessor {
    type In = WindowMessage;
    type Out = Event;

    fn dispatch(&mut self, _: &mut InputState, msg: WindowMessage) -> Option<Self::Out> {
        let position = msg.state.cursor_position().unwrap();
        let (state, event) = match (self.state.clone(), msg.event) {
            (State::Waiting, event) if event.left_button_pressed() => {
                (State::Pressed, Some(Event::Begin(position)))
            }
            (State::Pressed, WindowEvent::CursorMoved(current)) => {
                (State::Moved, Some(Event::Update(current)))
            }
            (State::Moved, WindowEvent::CursorMoved(current)) => {
                (State::Moved, Some(Event::Update(current)))
            }
            (State::Pressed, event) if event.left_button_released() => {
                (State::Waiting, Some(Event::Commit(position)))
            }
            (State::Moved, event) if event.left_button_released() => {
                (State::Waiting, Some(Event::Commit(position)))
            }
            (state, _) => (state, None),
        };
        self.state = state;
        event
    }
}
