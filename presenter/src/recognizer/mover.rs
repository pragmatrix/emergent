use crate::recognizer::pan;
use crate::{InputProcessor, InputState};
use emergent_drawing::{Point, Vector};
use emergent_ui::WindowMessage;
use std::marker::PhantomData;

// ID is the initialization data of the move transaction.
#[derive(Clone, Debug)]
pub enum Event<ID> {
    Begin(ID, Vector),
    Update(ID, Vector),
    Commit(ID, Vector),
    Rollback(ID),
}

impl<ID> Event<ID> {
    /// Returns the initialization data and the current moving vector.
    pub fn state(&self) -> (ID, Vector)
    where
        ID: Clone,
    {
        match self {
            Event::Begin(id, v) | Event::Update(id, v) | Event::Commit(id, v) => (id.clone(), *v),
            Event::Rollback(id) => ((*id).clone(), Vector::default()),
        }
    }

    pub fn is_active(&self) -> bool {
        match self {
            Event::Begin(_, _) => true,
            Event::Update(_, _) => true,
            Event::Commit(_, _) => false,
            Event::Rollback(_) => false,
        }
    }
}

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
    type Out = Event<ID>;

    fn dispatch(
        &mut self,
        input_state: &mut InputState,
        message: WindowMessage,
    ) -> Option<Self::Out> {
        // TODO: implement rollback.
        let e = self.pan.dispatch(input_state, message)?;
        let state: &mut State = input_state.get_state()?;
        match e.into() {
            pan::Event::Begin(p) => {
                self.transaction = (self.init_f)(state, p);
                self.transaction
                    .clone()
                    .map(|id| Event::Begin(id, Vector::default()))
            }
            pan::Event::Moved(_, v) => {
                let id = self.transaction.as_mut().unwrap();
                Some(Event::Update(id.clone(), v))
            }
            pan::Event::End(_, v) => {
                let id = self.transaction.as_mut().unwrap().clone();
                self.transaction = None;
                Some(Event::Commit(id, v))
            }
        }
    }
}
