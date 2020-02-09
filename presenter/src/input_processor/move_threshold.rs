use crate::input_processor::transaction::Transaction;
use crate::{InputProcessor, InputState};
use emergent_drawing::{scalar, Point};

// TODO: define what threshold is about (points, pixels?)

pub trait WithMoveThreshold {
    fn with_move_threshold(self, threshold: scalar) -> MoveThreshold<Self>
    where
        Self: Sized,
    {
        MoveThreshold {
            processor: self,
            threshold,
            state: State::Idle,
        }
    }
}

impl<T> WithMoveThreshold for T where T: InputProcessor<Out = Transaction<Point>> {}

pub struct MoveThreshold<P> {
    processor: P,
    threshold: scalar,
    state: State,
}

#[derive(Copy, Clone, Debug)]
enum State {
    Idle,
    Begun(Point),
    ThresholdReached,
}

impl<P> InputProcessor for MoveThreshold<P>
where
    P: InputProcessor<Out = Transaction<Point>>,
{
    type In = P::In;
    type Out = P::Out;

    fn dispatch(&mut self, input_state: &mut InputState, message: Self::In) -> Option<Self::Out> {
        let e = self.processor.dispatch(input_state, message)?;
        use State::*;
        use Transaction::*;
        let (e, new_state) = match (e, self.state) {
            (Begin(p), _) => (None, Begun(p)),
            (Update(p), Begun(p_start)) => {
                if (p - p_start).length() >= self.threshold {
                    (Some(Begin(p_start)), ThresholdReached)
                } else {
                    (None, Begun(p_start))
                }
            }
            (e, state @ ThresholdReached) => {
                let state = if e.is_active() { state } else { Idle };
                (Some(e), state)
            }
            (_, state) => (None, state),
        };

        self.state = new_state;
        e
    }
}
