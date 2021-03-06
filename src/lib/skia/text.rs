use super::convert::ToSkia;
use crate::{text_as_lines, TextOrigin};
use emergent_drawing as drawing;
use emergent_drawing::functions::*;
use emergent_drawing::{Bounds, FastBounds, FromTestEnvironment, Text, Union};
use emergent_ui::DPI;
use skia_safe::{Font, Point, Rect, Shaper, Typeface};

// Primitive text measurement and text rendering.
pub struct PrimitiveText {
    dpi: DPI,
}

impl PrimitiveText {
    pub fn new(dpi: DPI) -> PrimitiveText {
        PrimitiveText { dpi }
    }
}

impl FromTestEnvironment for PrimitiveText {
    fn from_test_environment() -> PrimitiveText {
        PrimitiveText {
            dpi: DPI::from_test_environment(),
        }
    }
}

impl drawing::MeasureText for PrimitiveText {
    fn measure_text(&self, text: &Text) -> drawing::Bounds {
        let font = &text.font;
        let typeface = Typeface::from_name(&font.name, font.style.to_skia())
            .expect("failed to resolve typeface");
        let font = Font::from_typeface(&typeface, self.dpi.scale_font_points(*font.size) as f32);

        let mut origin = TextOrigin::new(text.origin);
        let mut combined = None;
        for run in &text.runs {
            let (new_origin, bounds) = self.measure_run(run, &font, origin);
            origin = new_origin;
            combined = combined.union_with(Some(bounds));
        }
        combined.expect("empty text")
    }
}

impl PrimitiveText {
    fn measure_run(
        &self,
        run: &drawing::text::Run,
        font: &Font,
        mut origin: TextOrigin,
    ) -> (TextOrigin, Bounds) {
        use drawing::text::Run::*;
        let (line_spacing, metrics) = font.metrics();
        let line_spacing = line_spacing as drawing::scalar;
        match run {
            // TODO: support font style properties, these might affect measurements!
            Text(str, _properties) => {
                let mut combined = None;
                let mut last_line_advance = 0.0;

                for (i, line) in text_as_lines(str).enumerate() {
                    if i != 0 {
                        origin.newline(line_spacing)
                    }
                    let (advance, rect) = font.measure_str(line, None);
                    let width = rect.width();
                    // top & height are taken from the font metrics.
                    let bounds = bounds(
                        (
                            rect.left as drawing::scalar,
                            metrics.ascent as drawing::scalar,
                        ),
                        (
                            width as drawing::scalar,
                            (-metrics.ascent + metrics.descent) as drawing::scalar,
                        ),
                    ) + origin.point().to_vector();

                    combined = combined.union_with(Some(bounds));
                    last_line_advance = advance as drawing::scalar;
                }

                origin.advance(last_line_advance);

                (origin, combined.expect("empty run"))
            }
            Block(_) => unimplemented!(),
            Drawing(_, _) => unimplemented!(),
        }
    }
}

// TODO: Use a font cache that is shared between rendering and measuring.
// TODO: Implement a LRU measure cache for text.
pub struct MeasureWithShaper {
    pub dpi: DPI,
    pub shaper: Shaper,
}

impl MeasureWithShaper {
    pub fn new(dpi: DPI) -> MeasureWithShaper {
        MeasureWithShaper {
            dpi,
            shaper: Shaper::new(None),
        }
    }

    pub fn new_primitive(dpi: DPI) -> MeasureWithShaper {
        MeasureWithShaper {
            dpi,
            shaper: Shaper::new_primitive(),
        }
    }
}

impl drawing::MeasureText for MeasureWithShaper {
    fn measure_text(&self, text: &Text) -> drawing::Bounds {
        let font = &text.font;
        let typeface = Typeface::from_name(&font.name, font.style.to_skia())
            .expect("failed to resolve typeface");
        let font = Font::from_typeface(&typeface, self.dpi.scale_font_points(*font.size) as f32);

        // if there is no text we return the bounds of the text's origin point for now.
        measure_text_runs(&self.shaper, &font, text.origin, &text.runs)
            .unwrap_or_else(|| text.origin.fast_bounds())
    }
}

pub fn measure_text_runs(
    shaper: &Shaper,
    font: &Font,
    origin: drawing::Point,
    runs: &[drawing::text::Run],
) -> Option<drawing::Bounds> {
    let mut origin = origin.to_skia();
    let mut bounds: Option<drawing::Bounds> = None;
    for run in runs {
        let (run_bounds, new_origin) = measure_text_run(shaper, &font, origin, run);
        origin = new_origin;
        bounds = bounds.union_with(run_bounds);
    }
    bounds
}

fn measure_text_run(
    shaper: &Shaper,
    font: &Font,
    origin: Point,
    run: &drawing::text::Run,
) -> (Option<drawing::Bounds>, Point) {
    use drawing::text::Run::*;
    match run {
        Text(text, _properties) => {
            let (text_blob, end_point) =
                // TODO: support max width, right to left / bidi text..
                shaper.shape_text_blob(text, font, true, 100_000.0, origin).unwrap();
            // TODO: handle empty bounds returned from Skia here?
            trace!("text: {}", text);
            trace!("origin: {:?}", origin);
            trace!("bounds: {:?}", text_blob.bounds());
            trace!("end_point: {:?}", end_point);

            (Some(skia_rect_to_bounds(text_blob.bounds())), end_point)
        }
        Block(_) => unimplemented!("text::Run::Block"),
        Drawing(_, _) => unimplemented!("text::Run::Drawing"),
    }
}

fn skia_rect_to_bounds(rect: &Rect) -> drawing::Bounds {
    let (width, height) = (rect.size().width, rect.size().height);
    bounds(
        (rect.left as drawing::scalar, rect.top as drawing::scalar),
        (width as drawing::scalar, height as drawing::scalar),
    )
}
