use crate::{Clip, Drawing, DrawingBounds, IntoDrawing, IntoShape, MeasureText};

/// A trait to produce a drawing that visualizes something.
///
/// The result should be a drawing that is intended to visually explain what the object is.
///
/// For simple visualization, it's recommended to keep the paint in the drawing unset, so
/// that callers are able to parameterize the primary color.
pub trait Visualize {
    fn visualize(&self, measure: &dyn MeasureText) -> Drawing;
}

impl Visualize for Clip {
    fn visualize(&self, _: &dyn MeasureText) -> Drawing {
        match self {
            Clip::Rect(rect) => rect.clone().into_shape(),
            Clip::RoundedRect(rrect) => rrect.clone().into_shape(),
            Clip::Path(path) => path.clone().into_shape(),
        }
        .into_drawing()
    }
}

impl Visualize for DrawingBounds {
    fn visualize(&self, _: &dyn MeasureText) -> Drawing {
        match self {
            DrawingBounds::Empty | DrawingBounds::Unbounded => Drawing::Empty,
            DrawingBounds::Bounded(bounds) => bounds.to_rect().into_shape().into_drawing(),
        }
    }
}
