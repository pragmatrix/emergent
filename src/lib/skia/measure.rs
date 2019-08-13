use super::convert::ToSkia;
use emergent_drawing as drawing;
use emergent_drawing::functions::*;
use emergent_drawing::{FastBounds, Text, Union};
use skia_safe::{Font, Point, Rect, Shaper, Typeface};

// TODO: Use a font cache that is shared between rendering and measuring.
// TODO: Implement a LRU measure cache for text.
pub struct Measure {
    shaper: Shaper,
}

impl Measure {
    pub fn new() -> Measure {
        Measure {
            shaper: Shaper::new(),
        }
    }
}

impl drawing::MeasureText for Measure {
    fn measure_text(&self, text: &Text) -> drawing::Bounds {
        let font = &text.font;
        let typeface = Typeface::from_name(&font.name, font.style.to_skia())
            .expect("failed to resolve typeface");
        let font = Font::from_typeface(&typeface, *font.size as f32);

        // if there is no text we return the bounds of the text's origin point for now.
        measure_text_runs(&self.shaper, &text.text, &font, text.origin, &text.runs)
            .unwrap_or_else(|| text.origin.fast_bounds())

        /*
        let (_advance_width, rect) = font.measure_str(&text.text, None);
        let (width, height) = (rect.size().width, rect.size().height);
        bounds(
            (rect.left as drawing::scalar, rect.top as drawing::scalar),
            (width as drawing::scalar, height as drawing::scalar),
        ) + text.origin.to_vector()
        */
    }
}

fn measure_text_runs(
    shaper: &Shaper,
    text: &str,
    font: &Font,
    origin: drawing::Point,
    runs: &[drawing::text::Run],
) -> Option<drawing::Bounds> {
    if runs.is_empty() {
        measure_text_run(
            shaper,
            text,
            font,
            origin.to_skia(),
            &drawing::text::Run::Text(0..text.len(), drawing::text::properties()),
        )
        .0
    } else {
        let mut origin = origin.to_skia();
        let mut bounds: Option<drawing::Bounds> = None;
        for run in runs {
            let (run_bounds, new_origin) = measure_text_run(shaper, &text, &font, origin, run);
            origin = new_origin;
            bounds = Union::union(bounds, run_bounds);
        }
        bounds
    }
}

fn measure_text_run(
    shaper: &Shaper,
    text: &str,
    font: &Font,
    origin: Point,
    run: &drawing::text::Run,
) -> (Option<drawing::Bounds>, Point) {
    match run {
        drawing::text::Run::Text(range, properties) => {
            let (text_blob, end_point) =
                // TODO: support max width, right to left / bidi text..
                shaper.shape_text_blob(&text[range.clone()], font, true, std::f32::INFINITY, origin).unwrap();
            // TODO: handle empty bounds returned from Skia here?
            (Some(skia_rect_to_bounds(text_blob.bounds())), end_point)
        }
        drawing::text::Run::EndOfLine => {
            dbg!("unimplemented: EndOfLine");
            (None, origin)
        }
        drawing::text::Run::Block(_) => unimplemented!("text::Run::Block"),
        drawing::text::Run::Drawing(_, _) => unimplemented!("text::Run::Drawing"),
    }
}

fn skia_rect_to_bounds(rect: &Rect) -> drawing::Bounds {
    let (width, height) = (rect.size().width, rect.size().height);
    bounds(
        (rect.left as drawing::scalar, rect.top as drawing::scalar),
        (width as drawing::scalar, height as drawing::scalar),
    )
}
