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
pub use transform::*;

#[derive(Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct Drawing(pub Vec<Draw>);

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
