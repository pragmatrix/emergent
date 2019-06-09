use crate::{scalar, BlendMode, Color, Paint, PaintStyle, StrokeCap};

impl Paint {
    pub fn new() -> Self {
        Paint::default()
    }

    pub fn style(&mut self, style: PaintStyle) -> &mut Self {
        self.style = Some(style);
        self
    }

    pub fn color(&mut self, color: impl Into<Color>) -> &mut Self {
        self.color = Some(color.into());
        self
    }

    pub fn stroke_width(&mut self, width: scalar) -> &mut Self {
        self.stroke_width = Some(width);
        self
    }

    pub fn stroke_miter(&mut self, miter: scalar) -> &mut Self {
        self.stroke_miter = Some(miter);
        self
    }

    pub fn stroke_cap(&mut self, cap: StrokeCap) -> &mut Self {
        self.stroke_cap = Some(cap);
        self
    }

    pub fn stroke_join(&mut self, join: StrokeCap) -> &mut Self {
        self.stroke_cap = Some(join);
        self
    }

    pub fn blend_mode(&mut self, blend_mode: BlendMode) -> &mut Self {
        self.blend_mode = Some(blend_mode);
        self
    }
}
