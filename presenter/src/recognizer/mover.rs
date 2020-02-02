use crate::recognizer::pan::Event;
use crate::recognizer::PanRecognizer;
use crate::{GestureRecognizer, InputState};
use emergent_drawing::{Point, Vector};
use emergent_ui::WindowMessage;
use std::marker::PhantomData;

pub trait MoveTransaction<Msg> {
    type State;

    fn update(&mut self, pos: Vector, s: &mut Self::State) -> Option<Msg>;
    fn commit(&mut self, pos: Vector, s: &mut Self::State) -> Option<Msg>;
    fn rollback(&mut self, s: &mut Self::State) -> Option<Msg>;
}

pub struct MoverRecognizer<Msg, IF, T>
where
    T: MoveTransaction<Msg>,
{
    pan: PanRecognizer,
    init_f: IF,
    transaction: Option<T>,
    pd: PhantomData<*const Msg>,
}

impl<Msg, IF, T> MoverRecognizer<Msg, IF, T>
where
    IF: Fn(&T::State, Point) -> Option<T>,
    T: MoveTransaction<Msg>,
    T::State: 'static,
{
    pub fn new(init_f: IF) -> impl GestureRecognizer<Event = Msg> {
        Self {
            pan: PanRecognizer::new(),
            init_f,
            transaction: None,
            pd: PhantomData,
        }
    }
}

impl<Msg, IF, T> GestureRecognizer for MoverRecognizer<Msg, IF, T>
where
    IF: Fn(&T::State, Point) -> Option<T>,
    T: MoveTransaction<Msg>,
    T::State: 'static,
{
    type Event = Msg;

    fn dispatch(
        &mut self,
        input_state: &mut InputState,
        message: WindowMessage,
    ) -> Option<Self::Event> {
        let e = self.pan.dispatch(input_state, message)?;
        let state: &mut T::State = input_state.get_mut()?;
        match e {
            Event::Pressed(p) => {
                assert!(self.transaction.is_none());
                self.transaction = (self.init_f)(state, p);
                None
            }
            Event::Moved(_, v) => self.transaction.as_mut().unwrap().update(v, state),
            Event::Released(_, v) => {
                let m = self.transaction.as_mut().unwrap().commit(v, state);
                self.transaction = None;
                m
            }
        }
    }
}
