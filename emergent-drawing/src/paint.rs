use crate::{scalar, BlendMode, Color, Paint, PaintStyle, StrokeCap, StrokeJoin};

impl Default for Paint {
    fn default() -> Self {
        Paint::new()
    }
}

impl Paint {
    pub(crate) const DEFAULT: Paint = Paint::new();

    pub const fn new() -> Self {
        Self {
            style: PaintStyle::Fill,
            color: Color(0xff000000),
            stroke_width: 0.0,
            stroke_miter: 4.0,
            stroke_cap: StrokeCap::Butt,
            stroke_join: StrokeJoin::Miter,
            blend_mode: BlendMode::Source,
        }
    }

    pub fn style(&mut self, style: PaintStyle) -> &mut Self {
        self.style = style;
        self
    }

    pub fn color(&mut self, color: impl Into<Color>) -> &mut Self {
        self.color = color.into();
        self
    }

    pub fn stroke_width(&mut self, width: scalar) -> &mut Self {
        self.stroke_width = width;
        self
    }

    pub fn stroke_miter(&mut self, miter: scalar) -> &mut Self {
        self.stroke_miter = miter;
        self
    }

    pub fn stroke_cap(&mut self, cap: StrokeCap) -> &mut Self {
        self.stroke_cap = cap;
        self
    }

    pub fn stroke_join(&mut self, join: StrokeCap) -> &mut Self {
        self.stroke_cap = join;
        self
    }

    pub fn blend_mode(&mut self, blend_mode: BlendMode) -> &mut Self {
        self.blend_mode = blend_mode;
        self
    }
}

//
// Serialization Helper
//

impl Paint {
    pub(crate) fn is_style_default(style: &PaintStyle) -> bool {
        *style == Self::DEFAULT.style
    }

    pub(crate) fn is_color_default(color: &Color) -> bool {
        *color == Self::DEFAULT.color
    }

    pub(crate) fn is_stroke_width_default(width: &scalar) -> bool {
        *width == Self::DEFAULT.stroke_width
    }

    pub(crate) fn is_stroke_miter_default(miter: &scalar) -> bool {
        *miter == Self::DEFAULT.stroke_miter
    }

    pub(crate) fn is_stroke_cap_default(cap: &StrokeCap) -> bool {
        *cap == Self::DEFAULT.stroke_cap
    }

    pub(crate) fn is_stroke_join_default(join: &StrokeJoin) -> bool {
        *join == Self::DEFAULT.stroke_join
    }

    pub(crate) fn is_blend_mode_default(mode: &BlendMode) -> bool {
        *mode == Self::DEFAULT.blend_mode
    }

    pub(crate) fn default_style() -> PaintStyle {
        Self::DEFAULT.style
    }

    pub(crate) fn default_color() -> Color {
        Self::DEFAULT.color
    }

    pub(crate) fn default_stroke_width() -> scalar {
        Self::DEFAULT.stroke_width
    }

    pub(crate) fn default_stroke_miter() -> scalar {
        Self::DEFAULT.stroke_miter
    }

    pub(crate) fn default_stroke_cap() -> StrokeCap {
        Self::DEFAULT.stroke_cap
    }

    pub(crate) fn default_stroke_join() -> StrokeJoin {
        Self::DEFAULT.stroke_join
    }

    pub(crate) fn default_blend_mode() -> BlendMode {
        Self::DEFAULT.blend_mode
    }
}
