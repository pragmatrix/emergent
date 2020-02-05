use crate::recognizer::pan;
use crate::recognizer::PanRecognizer;
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

pub struct MoverRecognizer<IF, State, ID> {
    pan: PanRecognizer,
    init_f: IF,
    transaction: Option<ID>,
    pd: PhantomData<*const State>,
}

impl<IF, State, ID> MoverRecognizer<IF, State, ID>
where
    IF: Fn(&State, Point) -> Option<ID>,
    State: 'static,
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

impl<IF, State, ID> InputProcessor for MoverRecognizer<IF, State, ID>
where
    IF: Fn(&State, Point) -> Option<ID>,
    State: 'static,
    ID: Clone,
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
        match e {
            pan::Event::Pressed(p) => {
                assert!(self.transaction.is_none());
                self.transaction = (self.init_f)(state, p);
                self.transaction
                    .clone()
                    .map(|id| Event::Begin(id, Vector::default()))
            }
            pan::Event::Moved(_, v) => {
                let id = self.transaction.as_mut().unwrap();
                Some(Event::Update(id.clone(), v))
            }
            pan::Event::Released(_, v) => {
                let id = self.transaction.as_mut().unwrap().clone();
                self.transaction = None;
                Some(Event::Commit(id, v))
            }
        }
    }
}
