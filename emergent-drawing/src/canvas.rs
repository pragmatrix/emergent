//! Usability optimized drawing functions and wrappers.

use crate::drawing_target::DrawingTarget;
use crate::{
    scalar, Arc, Circle, Line, Oval, Paint, Path, Point, Polygon, Radius, Rect, RoundedRect, Shape,
    Size,
};

pub struct Canvas<'a, DT>
where
    DT: DrawingTarget,
{
    drawing_target: &'a mut DT,
}

impl<'a, DT: DrawingTarget> Canvas<'a, DT> {
    pub fn from_target(drawing_target: &'a mut DT) -> Self {
        Canvas { drawing_target }
    }

    pub fn draw<IS: Into<Shape>>(&mut self, shape: IS, paint: &Paint) -> &mut Self {
        self.drawing_target.draw(&shape.into(), paint);
        self
    }
}
