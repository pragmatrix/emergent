use super::convert::ToSkia;
use emergent_drawing as drawing;
use emergent_drawing::functions::*;
use skia_safe::{Font, Typeface};

// TODO: Use a font cache that is shared between rendering and measuring.
// TODO: Implement a LRU measure cache for text.
pub struct Measure;

impl Measure {
    pub fn new() -> Measure {
        Measure
    }
}

impl drawing::MeasureText for Measure {
    fn measure_text(&self, str: &str, font: &drawing::Font) -> drawing::Bounds {
        let typeface = Typeface::from_name(&font.name, font.style.to_skia())
            .expect("failed to resolve typeface");
        let font = Font::from_typeface(&typeface, *font.size as f32);
        let (_advance_width, rect) = font.measure_str(str, None);
        let (width, height) = (rect.size().width, rect.size().height);
        bounds(
            (rect.left as drawing::scalar, rect.top as drawing::scalar),
            (width as drawing::scalar, height as drawing::scalar),
        )
    }
}
