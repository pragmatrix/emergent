use crate::recognizer::pan;
use crate::recognizer::PanRecognizer;
use crate::{InputProcessor, InputState};
use emergent_drawing::{Point, Vector};
use emergent_ui::WindowMessage;
use std::marker::PhantomData;

pub trait MoveTransaction<Msg> {
    type State;

    fn update(&mut self, pos: Vector, s: &mut Self::State) -> Option<Msg>;
    fn commit(&mut self, pos: Vector, s: &mut Self::State) -> Option<Msg>;
    fn rollback(&mut self, s: &mut Self::State) -> Option<Msg>;
}

pub enum Event {
    Update(Vector),
    Commit(Vector),
    Rollback,
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
    pub fn new(init_f: IF) -> Self {
        Self {
            pan: PanRecognizer::new(),
            init_f,
            transaction: None,
            pd: PhantomData,
        }
    }

    pub fn is_active(&self) -> bool {
        self.transaction.is_some()
    }
}

impl<Msg, IF, T> InputProcessor for MoverRecognizer<Msg, IF, T>
where
    IF: Fn(&T::State, Point) -> Option<T>,
    T: MoveTransaction<Msg>,
    T::State: 'static,
{
    type In = WindowMessage;
    type Out = Msg;

    fn dispatch(
        &mut self,
        input_state: &mut InputState,
        message: WindowMessage,
    ) -> Option<Self::Out> {
        let e = self.pan.dispatch(input_state, message)?;
        let state: &mut T::State = input_state.get_state()?;
        match e {
            pan::Event::Pressed(p) => {
                assert!(self.transaction.is_none());
                self.transaction = (self.init_f)(state, p);
                None
            }
            pan::Event::Moved(_, v) => self.transaction.as_mut().unwrap().update(v, state),
            pan::Event::Released(_, v) => {
                let m = self.transaction.as_mut().unwrap().commit(v, state);
                self.transaction = None;
                m
            }
        }
    }
}
