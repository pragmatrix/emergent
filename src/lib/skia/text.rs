use super::convert::ToSkia;
use crate::skia::generation_cache::GenerationCache;
use crate::{text_as_lines, TextOrigin};
use emergent_drawing as drawing;
use emergent_drawing::functions::*;
use emergent_drawing::{Bounds, FastBounds, FromTestEnvironment, Text, Union};
use emergent_ui::DPI;
use skia_safe::typeface::FontId;
use skia_safe::{Font, Point, Rect, Shaper, Typeface};
use std::borrow::Borrow;
use std::hash::{Hash, Hasher};

// Primitive text measurement and text rendering.
pub struct PrimitiveText {
    dpi: DPI,
    cache: GenerationCache<TextMeasureKey, TextMeasureResult>,
}

impl PrimitiveText {
    pub const CACHE_MAX_GENERATIONS: usize = 10;

    pub fn new(dpi: DPI) -> PrimitiveText {
        PrimitiveText {
            dpi,
            cache: GenerationCache::new(Self::CACHE_MAX_GENERATIONS),
        }
    }
}

impl FromTestEnvironment for PrimitiveText {
    fn from_test_environment() -> PrimitiveText {
        Self::new(DPI::from_test_environment())
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
                    // let (advance, rect) = font.measure_str(line, None);
                    let (advance, rect) = self.measure_str(font, line);
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

    fn measure_str(&self, font: &Font, str: &str) -> (f32, Rect) {
        let key = (
            font.typeface().unwrap().unique_id(),
            (font.size() * 72.0) as i32,
            str,
        );
        self.cache.resolve(&key as &dyn TextMeasureKeyTuple, || {
            font.measure_str(str, None)
        })
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

#[derive(Clone, PartialEq, Eq, Hash)]
struct TextMeasureKey(FontId, i32, String);

type TextMeasureResult = (f32, Rect);

trait TextMeasureKeyTuple {
    fn font_id(&self) -> &FontId;
    fn size(&self) -> &i32;
    fn str(&self) -> &str;
}

impl<'a> Borrow<dyn TextMeasureKeyTuple + 'a> for TextMeasureKey {
    fn borrow(&self) -> &(dyn TextMeasureKeyTuple + 'a) {
        self
    }
}

impl<'a> Borrow<dyn TextMeasureKeyTuple + 'a> for (FontId, i32, &'a str) {
    fn borrow(&self) -> &(dyn TextMeasureKeyTuple + 'a) {
        self
    }
}

impl TextMeasureKeyTuple for (u32, i32, &'_ str) {
    fn font_id(&self) -> &u32 {
        &self.0
    }

    fn size(&self) -> &i32 {
        &self.1
    }

    fn str(&self) -> &str {
        &self.2
    }
}

impl TextMeasureKeyTuple for TextMeasureKey {
    fn font_id(&self) -> &u32 {
        &self.0
    }

    fn size(&self) -> &i32 {
        &self.1
    }

    fn str(&self) -> &str {
        &self.2
    }
}

impl Hash for dyn TextMeasureKeyTuple + '_ {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.font_id().hash(state);
        self.size().hash(state);
        self.str().hash(state);
    }
}

impl PartialEq for dyn TextMeasureKeyTuple + '_ {
    fn eq(&self, other: &Self) -> bool {
        self.font_id() == other.font_id()
            && self.size() == other.size()
            && self.str() == other.str()
    }
}

impl Eq for dyn TextMeasureKeyTuple + '_ {}

impl ToOwned for dyn TextMeasureKeyTuple + '_ {
    type Owned = TextMeasureKey;

    fn to_owned(&self) -> Self::Owned {
        TextMeasureKey(*self.font_id(), *self.size(), self.str().to_owned())
    }
}

#[test]
fn test_cache_key_same_entries() {
    let mut cache = GenerationCache::<TextMeasureKey, TextMeasureResult>::new(100);
    assert_eq!(cache.len(), 0);

    let measure = || ((0.4), Rect::new(10.0, 11.0, 12.0, 13.0));
    let measure_tuple = (10u32, 11i32, "Hello");
    let _ = cache.resolve(&measure_tuple as &dyn TextMeasureKeyTuple, measure);
    assert_eq!(cache.len(), 1);
    let measure_tuple2 = (10u32, 11i32, "Hello");
    let measure2 = || panic!("unexpected");
    let _ = cache.resolve(&measure_tuple2 as &dyn TextMeasureKeyTuple, measure2);
    assert_eq!(cache.len(), 1);
}
