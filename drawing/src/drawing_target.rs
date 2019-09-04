//! Function based API to specify drawings.
use crate::{BlendMode, Clip, Drawing, Paint, Shape, Transform};

pub mod drawing;

/// A drawing target is a function based API for drawing commands.
///
/// These are the essential commands that a drawing target must be able to
/// process to provide all the functionality to implement a canvas.
pub trait DrawingTarget {
    /// Fill all the available area with the Paint.
    fn fill(&mut self, paint: Paint, blend_mode: BlendMode);
    /// Draw a shape.
    fn draw_shape(&mut self, shape: &Shape, paint: Paint);
    /// Intersect clip with the current clipping area and draw a nested drawing.
    fn clip(&mut self, clip: &Clip, f: impl FnOnce(&mut Self));
    /// Apply the matrix transformation to the current matrix and draw a nested drawing.
    fn transform(&mut self, transformation: &Transform, f: impl FnOnce(&mut Self));

    /// Draw something that can be converted into a shape.
    fn draw(&mut self, shape: impl Into<Shape>, paint: Paint) {
        self.draw_shape(&shape.into(), paint)
    }
}

/// A trait for something that is drawable to a drawing target.
pub trait DrawTo {
    fn draw_to(&self, current_paint: Paint, target: &mut impl DrawingTarget);
}

impl DrawTo for Drawing {
    fn draw_to(&self, current_paint: Paint, target: &mut impl DrawingTarget) {
        use Drawing::*;
        match self {
            Empty => {}
            WithPaint(paint, drawing) => drawing.draw_to(*paint, target),
            Transformed(transform, drawing) => {
                target.transform(transform, |dt| drawing.draw_to(current_paint, dt))
            }
            Clipped(clip, drawing) => target.clip(clip, |dt| drawing.draw_to(current_paint, dt)),
            BackToFront(drawings) => drawings
                .iter()
                .for_each(|d| d.draw_to(current_paint, target)),
            Fill(blend_mode) => target.fill(current_paint, *blend_mode),
            Shape(shape) => target.draw_shape(shape, current_paint),
        }
    }
}
