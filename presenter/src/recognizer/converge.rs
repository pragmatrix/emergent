/*
use crate::recognizer::{momentum, Translate};
use crate::{InputProcessor, InputState};
use emergent_drawing::{scalar, Point, Vector};
use emergent_ui::WindowMessage;

pub struct Converge<P, TF, EF> {
    processor: P,
    get_target: TF,
    easing: EF,
}

trait ConvergeSource: Translate {
    fn can_converge(&self) -> bool;
    fn absolute_pos(&self) -> Point;
    fn translate(self, v: Vector) -> Self;
}

impl ConvergeSource for momentum::Event {
    fn can_converge(&self) -> bool {
        self.phase() == momentum::Phase::Drifting
    }

    fn absolute_pos(&self) -> Point {
        self.pos()
    }

    fn translate(self, v: Vector) -> Self {
        <Self as Translate>::translate(self, v)
    }
}

pub trait ConvergeTo {
    fn converge_to<TF, EF>(self, get_target: TF, easing: EF) -> Converge<Self, TF, EF>
    where
        Self: Sized,
    {
        Converge {
            processor: self,
            get_target,
            easing,
        }
    }
}

impl<P, TF, EF, Source> InputProcessor for Converge<P, TF, EF>
where
    P: InputProcessor<In = WindowMessage, Out = Source> + Sized,
    Source: ConvergeSource,
{
    type In = WindowMessage;
    type Out = Source;

    fn dispatch(&mut self, input_state: &mut InputState, message: Self::In) -> Option<Self::Out> {
        let e = self.processor.dispatch(input_state, message)?;
    }
}
*/
