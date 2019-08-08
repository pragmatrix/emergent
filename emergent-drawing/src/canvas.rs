//! Usability optimized drawing functions and wrappers.

use crate::{DrawingTarget, Paint, Shape};

// TODO: consider removal.
pub trait Canvas<DT: DrawingTarget> {
    fn target(&mut self) -> &mut DT;

    fn draw(&mut self, shape: impl Into<Shape>, paint: &Paint) -> &mut Self {
        self.target().draw(shape, paint);
        self
    }
}
