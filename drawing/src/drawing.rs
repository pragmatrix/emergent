use crate::{BackToFront, Shape};
use serde::{Deserialize, Serialize};

mod blend_mode;
pub use blend_mode::*;

mod clip;
pub use clip::*;

mod color;
pub use color::*;

pub mod font;
pub use font::Font;

pub mod paint;
pub use paint::Paint;

mod transform;
pub use transform::*;

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Drawing {
    Empty,
    /// Draw the nested drawing with the current paint.
    WithPaint(Paint, Box<Drawing>),
    /// Draw a drawing transformed with the current matrix.
    Transformed(Transform, Box<Drawing>),
    /// Intersect the current clip with the given Clip and draw the nested drawing.
    Clipped(Clip, Box<Drawing>),
    /// Draw Drawings from back to front.
    BackToFront(Vec<Drawing>),
    /// Fill the current clipping area with the current paint and `BlendMode`.
    Fill(BlendMode),
    /// Draw the shape with the current `Clip`, `Transform` and `Paint`.
    Shape(Shape),
}

impl Default for Drawing {
    fn default() -> Self {
        Drawing::new()
    }
}

impl<T: IntoIterator<Item = Drawing>> BackToFront<Drawing> for T {
    fn back_to_front(self) -> Drawing {
        Drawing::BackToFront(self.into_iter().collect())
    }
}

impl Drawing {
    pub const fn new() -> Self {
        Drawing::Empty
    }

    /// Creates a drawing with the default paint set to `paint`.
    pub fn with_paint(self, paint: impl Into<Paint>) -> Self {
        let paint = paint.into();
        match self.default_paint() {
            Some(p) if *p == paint => self,
            _ => Drawing::WithPaint(paint, self.into()),
        }
    }

    /// Push a drawing in the front of the current drawing.
    pub fn below(self, topmost: Drawing) -> Self {
        use Drawing::*;
        match self {
            Empty => topmost,
            BackToFront(mut v) => {
                v.push(topmost);
                BackToFront(v)
            }
            drawing => BackToFront(vec![drawing, topmost]),
        }
    }

    pub fn above(self, below: Drawing) -> Self {
        below.below(self)
    }

    /// The default paint that is used for all drawings.
    ///
    /// Returns `None` if the drawing does not specify a default paint.
    pub fn default_paint(&self) -> Option<&Paint> {
        use Drawing::*;
        match self {
            WithPaint(paint, _) => Some(paint),
            Transformed(_, drawing) | Clipped(_, drawing) => drawing.default_paint(),
            _ => None,
        }
    }
}

impl Clipped for Drawing {
    fn clipped(self, clip: impl Into<Clip>) -> Self {
        Drawing::Clipped(clip.into(), self.into())
    }
}

impl Transformed for Drawing {
    /// Returns the drawing transformed.
    fn transformed(self, transform: Transform) -> Self {
        use Drawing::*;
        match self {
            Empty => Empty,
            Transformed(t, drawing) => match Transform::optimized(&t, &transform) {
                Some(optimized) => Transformed(optimized, drawing),
                None => Transformed(transform, Transformed(t, drawing).into()),
            },
            _ => Transformed(transform, self.into()),
        }
    }
}

impl From<Shape> for Drawing {
    fn from(shape: Shape) -> Self {
        Drawing::Shape(shape)
    }
}

pub trait IntoDrawing {
    fn into_drawing(self) -> Drawing;
}

impl<T: Into<Drawing>> IntoDrawing for T {
    fn into_drawing(self) -> Drawing {
        self.into()
    }
}
