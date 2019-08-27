use crate::length;
use emergent_drawing::Font;

/// A trait to represents the functions used for external measurements.
pub trait Measure {
    /// Returns the length of the text in the writing direction and the length in
    /// the line layout direction.
    ///
    /// If the maximum length is specified, glyphs _should_ not exceed the maximum
    /// length in the writing direction.
    ///
    /// If the maximum length is None, the text should be layouted without
    /// imposed wrapping.
    fn text(&self, text: &str, font: &Font, max: Option<length>) -> (length, length);
}

pub(crate) struct UnimplementedMeasure;

impl dyn Measure {
    pub fn unimplemented() -> impl Measure {
        UnimplementedMeasure
    }
}

impl Measure for UnimplementedMeasure {
    fn text(&self, _text: &str, _font: &Font, _max: Option<length>) -> (length, length) {
        unimplemented!()
    }
}
