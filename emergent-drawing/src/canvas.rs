//! Usability optimized drawing functions and wrappers.

use crate::drawing_target::DrawingTarget;
use crate::{
    Arc, Circle, Line, Oval, Paint, Path, Point, Polygon, Radius, Rect, RoundedRect, Shape, Size,
};

pub trait Canvas<DT: DrawingTarget> {
    fn target(&mut self) -> &mut DT;

    fn draw(&mut self, shape: impl Into<Shape>, paint: &Paint) -> &mut Self {
        self.target().draw(&shape.into(), paint);
        self
    }
}
