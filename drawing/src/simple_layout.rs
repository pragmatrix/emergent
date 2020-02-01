//! Simple layout modules that can layout all types horizontally and vertically that expose
//! a way to be transformed and implement DrawingFastBounds.

use crate::{DrawingBounds, DrawingFastBounds, MeasureText, Point, Transform, Transformed, Vector};

pub trait SimpleLayout: Transformed + DrawingFastBounds + Sized {
    fn layout_horizontally(
        things: impl IntoIterator<Item = Self>,
        measure_text: &dyn MeasureText,
    ) -> Vec<Self> {
        stacked(things, measure_text, Vector::new(1.0, 0.0))
    }

    fn layout_vertically(
        things: impl IntoIterator<Item = Self>,
        measure_text: &dyn MeasureText,
    ) -> Vec<Self> {
        stacked(things, measure_text, Vector::new(0.0, 1.0))
    }
}

impl<T: Transformed + DrawingFastBounds + Sized> SimpleLayout for T {}

/// Stack a number of things vertically based on fast_bounds() computation.
///
/// The direction to stack the drawings is computed from the delta vector
/// which is multiplied with the bounds before to compute the location of
/// the next drawing.
pub fn stacked<T>(
    things: impl IntoIterator<Item = T>,
    measure: &dyn MeasureText,
    d: impl Into<Vector>,
) -> Vec<T>
where
    T: Transformed + DrawingFastBounds,
{
    let d = d.into();
    let mut p = Point::default();
    let mut r = Vec::new();
    for thing in things {
        if let DrawingBounds::Bounded(b) = thing.fast_bounds(measure) {
            let align = -b.point.to_vector();
            let transform = Transform::Translate((p + align).to_vector());
            r.push(thing.transformed(transform));
            p += Vector::from(b.extent) * d
        }
    }
    r
}
