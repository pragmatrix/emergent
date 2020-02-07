use crate::recognizer::pan;
use crate::recognizer::transaction::Transaction;
use crate::{InputProcessor, InputState};
use emergent_drawing::Point;
use emergent_ui::WindowMessage;
use std::marker::PhantomData;

pub struct Mover<R, IF, State, ID> {
    pan: R,
    init_f: IF,
    transaction: Option<ID>,
    pd: PhantomData<*const State>,
}

impl<R, IF, State, ID> Mover<R, IF, State, ID>
where
    IF: Fn(&State, Point) -> Option<ID>,
{
    pub fn new(pan: R, init_f: IF) -> Self {
        Self {
            pan,
            init_f,
            transaction: None,
            pd: PhantomData,
        }
    }

    pub fn is_active(&self) -> bool {
        self.transaction.is_some()
    }
}

pub trait IntoMovement {
    fn into_movement<IF, State, ID>(self, init_f: IF) -> Mover<Self, IF, State, ID>
    where
        IF: Fn(&State, Point) -> Option<ID>,
        Self: Sized,
    {
        Mover::new(self, init_f)
    }
}

impl<PE, R> IntoMovement for R
where
    Self: InputProcessor<In = WindowMessage, Out = PE>,
    PE: Into<pan::Event>,
{
}

impl<R, IF, State, ID, PE> InputProcessor for Mover<R, IF, State, ID>
where
    R: InputProcessor<In = WindowMessage, Out = PE>,
    IF: Fn(&State, Point) -> Option<ID>,
    State: 'static,
    ID: Clone,
    PE: Into<pan::Event>,
{
    type In = WindowMessage;
    type Out = Transaction<ID>;

    fn dispatch(
        &mut self,
        input_state: &mut InputState,
        message: WindowMessage,
    ) -> Option<Self::Out> {
        use Transaction::*;
        // TODO: implement rollback.
        let e = self.pan.dispatch(input_state, message)?;
        // if state is not available, it makes no sense to continue at all.
        let state: &mut State = input_state.get_state()?;
        match e.into() {
            Begin(p) => {
                let new_t = (self.init_f)(state, p);
                self.transaction = new_t;
                self.transaction.as_ref().map(|id| Begin(id.clone()))
            }
            Update(_, v) => {
                let id = self.transaction.as_mut().unwrap();
                Some(Update(id.clone(), v))
            }
            Commit(_, v) => {
                let id = self.transaction.as_mut().unwrap().clone();
                self.transaction = None;
                Some(Commit(id, v))
            }
            Rollback(_) => {
                let id = self.transaction.as_mut().unwrap().clone();
                self.transaction = None;
                Some(Rollback(id))
            }
        }
    }
}
