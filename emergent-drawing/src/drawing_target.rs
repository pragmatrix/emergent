//! Function based API to specify drawings.
use crate::drawing::{
    BlendMode, Clip, Draw, Drawing, Paint, Polygon, Rect, Shape, Size, Transformation,
};
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
    fn paint(&mut self) -> DrawingScope<Self>;
    /// When PaintScope is going out of scope, drop_scope will be called.
    // TODO: this could be called directly in nested scope and mess
    //       everything up, can we do something about that?
    fn drop_scope(&mut self, begin: usize);
    fn clip(&mut self, clip: &Clip) -> DrawingScope<Self>;
    fn transform(&mut self, transformation: &Transformation) -> DrawingScope<Self>;
}

/// A trait for something that is drawable to a drawing target.
pub trait DrawTo {
    fn draw_to<DT: DrawingTarget>(&self, target: &mut DT);
}

/// A nested drawing scope.
/// TODO: this is highly specific for now, because I don't want to put a lifetime or
///       associated type on the DrawingTarget trait.
pub struct DrawingScope<'a, DT: DrawingTarget> {
    /// Index into the internal structure representing this scope.
    begin: usize,
    target: &'a mut DT,
}

impl<'a, DT: DrawingTarget> Drop for DrawingScope<'a, DT> {
    fn drop(&mut self) {
        self.target.drop_scope(self.begin);
    }
}

impl<'a, DT: DrawingTarget> Deref for DrawingScope<'a, DT> {
    type Target = DT;
    fn deref(&self) -> &DT {
        self.target
    }
}

impl<'a, DT: DrawingTarget> DerefMut for DrawingScope<'a, DT> {
    fn deref_mut(&mut self) -> &mut DT {
        self.target
    }
}

impl<'a, DT: DrawingTarget> DrawingScope<'a, DT> {
    pub fn from_index(target: &'a mut DT, begin: usize) -> DrawingScope<'a, DT> {
        DrawingScope { begin, target }
    }
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
    fn draw_to<DT: DrawingTarget>(&self, target: &mut DT) {
        // drawing a drawing _always_ introduces a new scope in the drawing
        // target to avoid changing state.
        let scope = &mut target.paint();
        self.0
            .iter()
            .for_each(|drawing| drawing.draw_to(scope.deref_mut()));
    }
}

impl DrawTo for Draw {
    fn draw_to<DT: DrawingTarget>(&self, target: &mut DT) {
        match self {
            Draw::Paint(paint, blend_mode) => target.fill(&paint, *blend_mode),
            Draw::Shapes(shapes, paint) => {
                // TODO: optimize paint reuse here?
                shapes.iter().for_each(|shape| target.draw(shape, &paint))
            }
            Draw::Drawing(drawing) => drawing.draw_to(target),
            Draw::Clipped(clip, drawing) => {
                let target = &mut target.clip(clip);
                drawing.draw_to(target.target)
            }
            Draw::Transformed(transform, drawing) => {
                let mut target = target.transform(transform);
                drawing.draw_to(target.target);
            }
        }
    }
}

impl DrawingTarget for Drawing {
    fn fill(&mut self, paint: &Paint, blend_mode: BlendMode) {
        self.0.push(Draw::Paint(paint.clone(), blend_mode));
    }

    fn draw(&mut self, shape: &Shape, paint: &Paint) {
        match self.0.last_mut() {
            Some(Draw::Shapes(shapes, p)) if p == paint => {
                shapes.push(shape.clone());
            }
            _ => self
                .0
                .push(Draw::Shapes(vec![shape.clone()], paint.clone())),
        }
    }

    fn paint(&mut self) -> DrawingScope<Self> {
        self.0.push(Draw::Drawing(Drawing::new()));
        DrawingScope {
            begin: self.0.len(),
            target: self,
        }
    }

    fn drop_scope(&mut self, begin: usize) {
        let nested = Drawing(self.0.drain(begin..).collect());
        if !nested.is_empty() {
            match self.0.last_mut().unwrap() {
                Draw::Drawing(d) => *d = nested,
                Draw::Clipped(_, d) => *d = nested,
                Draw::Transformed(_, d) => *d = nested,
                _ => {}
            }
        } else {
            self.0.pop();
        }
    }

    fn clip(&mut self, clip: &Clip) -> DrawingScope<Self> {
        self.0.push(Draw::Clipped(clip.clone(), Drawing::new()));
        DrawingScope {
            begin: self.0.len(),
            target: self,
        }
    }

    fn transform(&mut self, transformation: &Transformation) -> DrawingScope<Self> {
        self.0
            .push(Draw::Transformed(transformation.clone(), Drawing::new()));
        DrawingScope {
            begin: self.0.len(),
            target: self,
        }
    }
}
