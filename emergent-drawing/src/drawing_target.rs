//! Function based API to specify drawings.
use crate::{BlendMode, Clip, Draw, Drawing, Paint, Shape, Transformation};

pub mod drawing;

/// A drawing target is a function based API for drawing commands.
///
/// These are the essential commands that a drawing target must be able to
/// process to provide all the functionality to implement a canvas.
pub trait DrawingTarget: Sized {
    fn fill(&mut self, paint: &Paint, blend_mode: BlendMode);
    fn draw(&mut self, shape: &Shape, paint: &Paint);
    fn paint(&mut self, f: impl FnOnce(&mut Self));
    fn clip(&mut self, clip: &Clip, f: impl FnOnce(&mut Self));
    fn transform(&mut self, transformation: &Transformation, f: impl FnOnce(&mut Self));
}

/// A trait for something that is drawable to a drawing target.
pub trait DrawTo {
    fn draw_to(&self, target: &mut impl DrawingTarget);
}

impl Drawing {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl DrawTo for Drawing {
    fn draw_to(&self, target: &mut impl DrawingTarget) {
        // drawing a drawing _always_ introduces a new scope in the drawing
        // target to avoid changing state.
        target.paint(|dt| {
            self.0.iter().for_each(|drawing| drawing.draw_to(dt));
        });
    }
}

impl DrawTo for Draw {
    fn draw_to(&self, target: &mut impl DrawingTarget) {
        match self {
            Draw::Paint(paint, blend_mode) => target.fill(&paint, *blend_mode),
            Draw::Shapes(shapes, paint) => {
                // TODO: optimize paint reuse here?
                shapes.iter().for_each(|shape| target.draw(shape, &paint))
            }
            Draw::Drawing(drawing) => drawing.draw_to(target),
            Draw::Clipped(clip, drawing) => target.clip(clip, |dt| drawing.draw_to(dt)),
            Draw::Transformed(transform, drawing) => {
                target.transform(transform, |dt| drawing.draw_to(dt));
            }
        }
    }
}
