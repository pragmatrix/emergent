use crate::GestureRecognizer;
use emergent_drawing::{Point, Vector};
use emergent_ui::WindowMsg;

pub struct PanRecognizer<Msg> {
    state: State,
    f: Box<dyn Fn(Event) -> Msg>,
}

type State = Option<Event>;

pub enum Event {
    DownAt(Point),
    Moved(Point, Vector),
    Released(Point, Vector),
}

impl<Msg> PanRecognizer<Msg> {
    pub fn new(f: impl Fn(Event) -> Msg + 'static) -> Self {
        Self {
            state: None,
            f: Box::new(f),
        }
    }
}

impl<Msg: 'static> GestureRecognizer for PanRecognizer<Msg> {
    type Msg = Msg;

    fn update(&mut self, msg: WindowMsg) -> Option<Self::Msg> {
        unimplemented!()
    }
}
