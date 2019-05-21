//! Function based API to specify drawings.
use crate::drawing::{
    BlendMode, Clip, Drawing, Paint, Painting, Rect, Shape, Size, Transformation,
};
use crate::drawing::{Point, Polygon};
use std::ops::{Deref, DerefMut};

/// A drawing target is a function based API for drawing commands.
///
/// These are the essential commands that a drawing target must be able to
/// process to provide all the functionality to implement a canvas.
/// This is an internal API that is meant to be a full storage and execution
/// backend for a more usable Canvas API.
/// TODO: review the PaintScope implementation, scope with a function for
///       example.
pub trait DrawingTarget: Sized {
    fn fill(&mut self, paint: &Paint, blend_mode: BlendMode);
    fn draw(&mut self, shape: &Shape, paint: &Paint);
    fn paint(&mut self) -> PaintScope<Self>;
    /// When PaintScope is going out of scope, drop_scope will be called.
    // TODO: this could be called directly in nested scope and mess
    //       everything up, can we do something about that?
    fn drop_scope(&mut self, begin: usize);
    fn clip(&mut self, clip: &Clip);
    fn transform(&mut self, transformation: &Transformation);
}

/// A trait for something that is drawable to a drawing target.
pub trait DrawTo {
    fn draw_to<DT: DrawingTarget>(&self, target: &mut DT);
}

/// A nested painting scope.
/// TODO: this is highly specific for now, because I don't want to put a lifetime or
///       associated type on the DrawingTarget trait.
pub struct PaintScope<'a, DT: DrawingTarget> {
    /// Index into the internal structure representing this scope.
    begin: usize,
    target: &'a mut DT,
}

impl<'a, DT: DrawingTarget> Drop for PaintScope<'a, DT> {
    fn drop(&mut self) {
        self.target.drop_scope(self.begin);
    }
}

impl<'a, DT: DrawingTarget> Deref for PaintScope<'a, DT> {
    type Target = DT;
    fn deref(&self) -> &DT {
        self.target
    }
}

impl<'a, DT: DrawingTarget> DerefMut for PaintScope<'a, DT> {
    fn deref_mut(&mut self) -> &mut DT {
        self.target
    }
}

impl<'a, DT: DrawingTarget> PaintScope<'a, DT> {
    pub fn from_index(target: &'a mut DT, begin: usize) -> PaintScope<'a, DT> {
        PaintScope { begin, target }
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

impl DrawTo for Painting {
    fn draw_to<DT: DrawingTarget>(&self, target: &mut DT) {
        // drawing a painting _always_ introduces a new scope in the drawing
        // target to avoid changing state.
        let scope = &mut target.paint();
        self.0
            .iter()
            .for_each(|drawing| drawing.draw_to(scope.deref_mut()));
    }
}

impl DrawTo for Drawing {
    fn draw_to<DT: DrawingTarget>(&self, target: &mut DT) {
        match self {
            Drawing::Fill(paint, blend_mode) => target.fill(&paint, *blend_mode),
            Drawing::Draw(shapes, paint) => {
                // TODO: optimize paint reuse here?
                shapes.iter().for_each(|shape| target.draw(shape, &paint))
            }
            Drawing::Paint(painting) => painting.draw_to(target),
            Drawing::Clip(clip) => target.clip(clip),
            Drawing::Transform(transform) => target.transform(transform),
        }
    }
}

impl DrawingTarget for Painting {
    fn fill(&mut self, paint: &Paint, blend_mode: BlendMode) {
        self.0.push(Drawing::Fill(paint.clone(), blend_mode));
    }

    fn draw(&mut self, shape: &Shape, paint: &Paint) {
        match self.0.last_mut() {
            Some(Drawing::Draw(shapes, p)) if p == paint => {
                shapes.push(shape.clone());
            }
            _ => self
                .0
                .push(Drawing::Draw(vec![shape.clone()], paint.clone())),
        }
    }

    fn paint(&mut self) -> PaintScope<Self> {
        PaintScope {
            begin: self.0.len(),
            target: self,
        }
    }

    fn drop_scope(&mut self, begin: usize) {
        let nested_painting = Painting(self.0.drain(begin..).collect());
        if !nested_painting.is_empty() {
            self.0.push(Drawing::Paint(nested_painting))
        }
    }

    fn clip(&mut self, clip: &Clip) {
        self.0.push(Drawing::Clip(clip.clone()));
    }

    fn transform(&mut self, transformation: &Transformation) {
        self.0.push(Drawing::Transform(transformation.clone()));
    }
}
