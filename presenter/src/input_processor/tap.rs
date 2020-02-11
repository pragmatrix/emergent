use crate::input_processor::{Pan, Subscriber, Subscriptions, Transaction, WithMoveThreshold};
use crate::{InputProcessor, InputState};
use emergent_drawing::Point;
use emergent_ui::WindowMessage;

pub enum Tap {}

impl Tap {
    pub fn new() -> impl InputProcessor<In = WindowMessage, Out = Event> + Subscriber {
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
