//! Function based API to specify drawings.
use crate::{BlendMode, Clip, Draw, Drawing, Paint, Shape, Transform};

pub mod drawing;

/// A drawing target is a function based API for drawing commands.
///
/// These are the essential commands that a drawing target must be able to
/// process to provide all the functionality to implement a canvas.
pub trait DrawingTarget: Sized {
    /// Fill all the available area with the Paint.
    fn fill(&mut self, paint: &Paint, blend_mode: BlendMode);
    /// Draw a shape.
    fn draw(&mut self, shape: &Shape, paint: &Paint);
    /// Intersect clip with the current clipping area and draw a nested drawing.
    fn clip(&mut self, clip: &Clip, f: impl FnOnce(&mut Self));
    /// Apply the matrix transformation to the current matrix and draw a nested drawing.
    fn transform(&mut self, transformation: &Transform, f: impl FnOnce(&mut Self));
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
        self.0.iter().for_each(|draw| draw.draw_to(target));
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
            Draw::Clipped(clip, drawing) => target.clip(clip, |dt| drawing.draw_to(dt)),
            Draw::Transformed(transform, drawing) => {
                target.transform(transform, |dt| drawing.draw_to(dt));
            }
        }
    }
}
