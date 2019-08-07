use crate::Shape;
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

impl<I: Iterator<Item = Draw>> From<I> for Drawing {
    fn from(v: I) -> Self {
        Drawing(v.collect())
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
