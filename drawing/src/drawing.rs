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
use std::mem;
use std::ops::{Deref, DerefMut};
pub use transform::*;

#[derive(Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct Drawing(Vec<Draw>);

impl<I: IntoIterator<Item = Draw>> From<I> for Drawing {
    fn from(v: I) -> Self {
        Drawing(v.into_iter().collect())
    }
}

// TODO: is this appropriate?
impl Deref for Drawing {
    type Target = Vec<Draw>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Drawing {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Drawing {
    pub fn new() -> Drawing {
        Drawing(Vec::new())
    }

    pub fn take(&mut self) -> Vec<Draw> {
        mem::replace(&mut self.0, Vec::new())
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
        let mut r = Drawing::new();
        for drawing in drawings {
            match drawing.fast_bounds(measure_text) {
                DrawingBounds::Bounded(b) => {
                    let align = -b.point.to_vector();
                    let transform = Transform::Translate((p + align).to_vector());
                    r.push(Draw::Transformed(transform, drawing));
                    p += Vector::from(b.extent) * d
                }
                _ => {}
            }
        }
        r
    }
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Draw {
    /// Fill the current clipping area with the given paint and blend mode.
    Paint(Paint, BlendMode),

    /// Draw a number of shapes with the same paint.
    Shapes(Vec<Shape>, Paint),

    // TODO: Skia supports ClipOp::Difference, which I suppose is quite unusual.
    // TODO: Also Skia supports do_anti_alias for clipping.
    /// Intersect the current clip with the given Clip and draw the nested drawing.
    Clipped(Clip, Drawing),

    /// Draw a drawing transformed with the current matrix.
    Transformed(Transform, Drawing),
}