//! Function based API to specify drawings.
use crate::drawing::{
    BlendMode, Clip, Drawing, Paint, Painting, Rect, Shape, Size, Transformation,
};
pub use crate::drawing::{Line, LineSegments, Point, Polygon};
use std::ops::{Deref, DerefMut};

/// The essential commands that a drawing target must be able to run to provide all the canvas functionality.
/// This is an internal API that is meant to be a full storage and execution backend for a more usable Canvas API.
/// Most of the function consume their input for optimization.
pub trait DrawingTarget<'a> {
    type PaintScope: 'a + Drop + DerefMut<Target = Self>;

    fn fill(&mut self, paint: &Paint, blend_mode: BlendMode);
    fn draw(&mut self, shape: Shape, paint: &Paint);
    fn paint(&'a mut self) -> Self::PaintScope;
    fn clip(&mut self, clip: Clip);
    fn transform(&mut self, transformation: Transformation);
}

pub struct PaintScope<'a> {
    /// Index into the painting when the nested scope was generated.
    begin: usize,
    target: &'a mut Painting,
}

impl<'a> Deref for PaintScope<'a> {
    type Target = Painting;

    fn deref(&self) -> &Self::Target {
        self.target
    }
}

impl<'a> DerefMut for PaintScope<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.target
    }
}

impl<'a> Drop for PaintScope<'a> {
    fn drop(&mut self) {
        let index = self.begin;
        let nested_painting = Painting(self.0.drain(index..).collect());
        if !nested_painting.is_empty() {
            self.0.push(Drawing::Paint(nested_painting))
        }
    }
}

impl Painting {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl<'a> DrawingTarget<'a> for Painting {
    type PaintScope = PaintScope<'a>;

    fn fill(&mut self, paint: &Paint, blend_mode: BlendMode) {
        self.0.push(Drawing::Fill(paint.clone(), blend_mode));
    }

    fn draw(&mut self, shape: Shape, paint: &Paint) {
        match self.0.last_mut() {
            Some(Drawing::Draw(shapes, p)) if p == paint => {
                shapes.push(shape);
            }
            _ => self.0.push(Drawing::Draw(vec![shape], paint.clone())),
        }
    }

    fn paint(&'a mut self) -> Self::PaintScope {
        PaintScope {
            begin: self.0.len(),
            target: self,
        }
    }

    fn clip(&mut self, clip: Clip) {
        self.0.push(Drawing::Clip(clip));
    }

    fn transform(&mut self, transformation: Transformation) {
        self.0.push(Drawing::Transform(transformation));
    }
}
