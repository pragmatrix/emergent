use crate::input_processor::{Pan, Transaction, WithMoveThreshold};
use crate::{InputProcessor, InputState};
use emergent_drawing::Point;
use emergent_ui::WindowMessage;

pub enum Tap {}

impl Tap {
    pub fn new() -> impl InputProcessor<In = WindowMessage, Out = Event> {
        Pan::new_bare()
            .with_move_stop_threshold(10.0)
            // .max_time_to_commit(Duration::from_millis(250))
            .map(|e| match e {
                Transaction::Commit(p) => Some(Event::Tapped(p)),
                _ => None,
            })
    }
}

pub enum Event {
    Tapped(Point),
}

impl InputProcessor for Tap {
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
