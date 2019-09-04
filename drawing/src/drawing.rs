use crate::{DrawingBounds, DrawingFastBounds, MeasureText, Point, Shape, Vector};
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

impl Drawing {
    pub const fn new() -> Self {
        Drawing::Empty
    }

    /// Creates a drawing with the default paint set to `paint`.
    pub fn with_paint(self, paint: Paint) -> Self {
        match self.default_paint() {
            Some(p) if *p == paint => self,
            _ => Drawing::WithPaint(paint, self.into()),
        }
    }

    /// Returns the drawing transformed.
    pub fn transformed(self, transform: Transform) -> Self {
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

    pub fn clipped(self, clip: Clip) -> Self {
        Drawing::Clipped(clip, self.into())
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

    pub fn stack_h(drawings: Vec<Drawing>, measure_text: &dyn MeasureText) -> Drawing {
        Self::stack(drawings, measure_text, Vector::new(1.0, 0.0))
    }

    pub fn stack_v(drawings: Vec<Drawing>, measure_text: &dyn MeasureText) -> Drawing {
        Self::stack(drawings, measure_text, Vector::new(0.0, 1.0))
    }

    /// Stack a number of drawings vertically based on fast_bounds() computation.
    ///
    /// The direction to stack the drawings is computed from the delta vector
    /// which is multiplied with the bounds before to compute the location of
    /// the next drawing.
    pub fn stack(
        drawings: Vec<Drawing>,
        measure_text: &dyn MeasureText,
        d: impl Into<Vector>,
    ) -> Drawing {
        let d = d.into();
        let mut p = Point::default();
        let mut r = Vec::new();
        for drawing in drawings {
            match drawing.fast_bounds(measure_text) {
                DrawingBounds::Bounded(b) => {
                    let align = -b.point.to_vector();
                    let transform = Transform::Translate((p + align).to_vector());
                    r.push(Drawing::Transformed(transform, drawing.into()));
                    p += Vector::from(b.extent) * d
                }
                _ => {}
            }
        }
        Drawing::BackToFront(r)
    }
}
