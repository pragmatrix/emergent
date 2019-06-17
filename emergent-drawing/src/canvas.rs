//! Usability optimized drawing functions and wrappers.

use crate::drawing_target::DrawingTarget;
use crate::{Paint, Shape};

pub trait Canvas<DT: DrawingTarget> {
    fn target(&mut self) -> &mut DT;

    fn draw(&mut self, shape: impl Into<Shape>, paint: &Paint) -> &mut Self {
        self.target().draw(&shape.into(), paint);
        self
    }
}
