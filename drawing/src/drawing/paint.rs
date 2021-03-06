use crate::{scalar, BlendMode, Color, Outset};
use serde::{Deserialize, Serialize};

// Decided to make Paint a value by implementing Copy. The compiler will
// be able to optimize a lot of copies away and users of this API won't have to
// think about references and cloning anymore. Another strong indicator for making
// paint a value type is that there seems to be no need to modify it in place.
// ref: https://skia.org/user/api/SkPaint_Reference
// TODO: convert this to an enum with the cases Fill, Stroke, FillAndStroke?
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Paint {
    #[serde(
        skip_serializing_if = "Paint::is_style_default",
        default = "Paint::default_style"
    )]
    pub style: Style,
    #[serde(
        skip_serializing_if = "Paint::is_color_default",
        default = "Paint::default_color"
    )]
    pub color: Color,
    #[serde(
        skip_serializing_if = "Paint::is_stroke_width_default",
        default = "Paint::default_stroke_width"
    )]
    pub width: scalar,
    #[serde(
        skip_serializing_if = "Paint::is_stroke_miter_default",
        default = "Paint::default_stroke_miter"
    )]
    pub miter: scalar,
    #[serde(
        skip_serializing_if = "Paint::is_stroke_cap_default",
        default = "Paint::default_stroke_cap"
    )]
    pub cap: Cap,
    #[serde(
        skip_serializing_if = "Paint::is_stroke_join_default",
        default = "Paint::default_stroke_join"
    )]
    pub join: Join,
    #[serde(
        skip_serializing_if = "Paint::is_blend_mode_default",
        default = "Paint::default_blend_mode"
    )]
    pub blend_mode: BlendMode,
}

pub fn paint() -> Paint {
    Paint::new()
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Style {
    Stroke,
    Fill,
    FillAndStroke,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Cap {
    Butt,
    Round,
    Square,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Join {
    Miter,
    Round,
    Bevel,
}

impl Paint {
    pub(crate) const DEFAULT: Paint = Paint::new();

    /// TODO: rename to fill_black?
    pub const fn new() -> Self {
        Self {
            style: Style::Fill,
            color: Color::BLACK,
            width: 0.0,
            miter: 4.0,
            cap: Cap::Butt,
            join: Join::Miter,
            blend_mode: BlendMode::Source,
        }
    }

    /// Returns a paint with style `Style::Stroke` and width set to 1.
    pub const fn stroke(color: Color) -> Self {
        Self {
            style: Style::Stroke,
            color,
            width: 1.0,
            ..Self::new()
        }
    }

    /// Returns a colored paint with `Style::Fill`.
    pub const fn fill(color: Color) -> Self {
        Self {
            style: Style::Fill,
            color,
            ..Self::new()
        }
    }

    /// Returns a paint to fill and stroke.
    pub const fn fill_and_stroke(color: Color, width: scalar) -> Self {
        Self {
            style: Style::FillAndStroke,
            color,
            width,
            ..Self::new()
        }
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.color = color.into();
        self
    }

    /// Sets the stroke width. Does not enable stroking style.
    pub fn width(mut self, width: scalar) -> Self {
        self.width = width;
        self
    }

    /// Sets the stroke's miter limit. Does not enable stroking style.
    pub fn miter(mut self, miter: scalar) -> Self {
        self.miter = miter;
        self
    }

    pub fn cap(mut self, cap: Cap) -> Self {
        self.cap = cap;
        self
    }

    pub fn join(mut self, join: Join) -> Self {
        self.join = join;
        self
    }

    pub fn blend_mode(mut self, blend_mode: BlendMode) -> Self {
        self.blend_mode = blend_mode;
        self
    }

    /// Fast outset, an approximate area around a figure drawing with that paint.
    pub fn fast_outset(&self) -> Outset {
        if self.width == 0.0 {
            return Outset::EMPTY;
        }

        match self.style {
            Style::Fill => Outset::EMPTY,
            Style::Stroke | Style::FillAndStroke => Outset::from(self.width / 2.0),
        }
    }
}

pub mod traits {
    use crate::{Color, Paint};

    impl Default for Paint {
        fn default() -> Self {
            Paint::new()
        }
    }

    impl From<Color> for Paint {
        fn from(color: Color) -> Self {
            Paint::fill(color)
        }
    }
}

pub mod serialization_helper {
    use crate::paint::{Cap, Join, Style};
    use crate::{scalar, BlendMode, Color, Paint};

    #[allow(clippy::float_cmp)]
    #[allow(clippy::trivially_copy_pass_by_ref)]
    impl Paint {
        pub(crate) fn is_style_default(style: &Style) -> bool {
            *style == Self::DEFAULT.style
        }

        pub(crate) fn is_color_default(color: &Color) -> bool {
            *color == Self::DEFAULT.color
        }

        pub(crate) fn is_stroke_width_default(width: &scalar) -> bool {
            *width == Self::DEFAULT.width
        }

        pub(crate) fn is_stroke_miter_default(miter: &scalar) -> bool {
            *miter == Self::DEFAULT.miter
        }

        pub(crate) fn is_stroke_cap_default(cap: &Cap) -> bool {
            *cap == Self::DEFAULT.cap
        }

        pub(crate) fn is_stroke_join_default(join: &Join) -> bool {
            *join == Self::DEFAULT.join
        }

        pub(crate) fn is_blend_mode_default(mode: &BlendMode) -> bool {
            *mode == Self::DEFAULT.blend_mode
        }

        pub(crate) fn default_style() -> Style {
            Self::DEFAULT.style
        }

        pub(crate) fn default_color() -> Color {
            Self::DEFAULT.color
        }

        pub(crate) fn default_stroke_width() -> scalar {
            Self::DEFAULT.width
        }

        pub(crate) fn default_stroke_miter() -> scalar {
            Self::DEFAULT.miter
        }

        pub(crate) fn default_stroke_cap() -> Cap {
            Self::DEFAULT.cap
        }

        pub(crate) fn default_stroke_join() -> Join {
            Self::DEFAULT.join
        }

        pub(crate) fn default_blend_mode() -> BlendMode {
            Self::DEFAULT.blend_mode
        }
    }
}
