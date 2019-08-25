use crate::{font, Color, Drawing, Font, Paint, Point, Vector};
use serde::{Deserialize, Serialize};
use serde_tuple::*;

/// Text, described by a, the font, and an origin.
///
/// The origin is treated as the starting point on baseline where the text
/// will be rendered.
#[derive(Clone, Serialize_tuple, Deserialize_tuple, PartialEq, Debug)]
pub struct Text {
    pub font: Font,
    pub origin: Point,
    pub runs: Vec<Run>,
}

pub fn text(text: impl AsRef<str>, font: &Font, origin: impl Into<Option<Point>>) -> Text {
    let mut t = Text::new(font, origin.into().unwrap_or_default());
    t.text(text, ());
    t
}

impl Text {
    pub fn new(font: &Font, origin: Point) -> Self {
        Text {
            font: font.clone(),
            origin,
            runs: Vec::new(),
        }
    }
}

/// Text runs that describe formatted text fragments.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Run {
    /// Draw Text with the given properties.
    Text(String, Properties),

    /// Another text block. This can be used to render text on the same baseline
    /// with other fonts.
    Block(Text),

    /// Render a drawing on the current text's baseline with an additional translation.
    Drawing(Drawing, Vector),
}

/// Creates a new block of text.
pub fn block(font: &Font, origin: impl Into<Option<Point>>) -> Text {
    Text {
        runs: Vec::new(),
        font: font.clone(),
        origin: origin.into().unwrap_or_default(),
    }
}

impl Text {
    /// Add a text run with the given properties.
    pub fn text(&mut self, text: impl AsRef<str>, properties: impl Into<Properties>) -> &mut Self {
        let text = text.as_ref();
        self.runs
            .push(Run::Text(String::from(text), properties.into()));
        self
    }
}

/// Properties that affect how text should be rendered.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct Properties {
    /// A color for text. If not set, uses the color of the current paint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    /// The fot style of the text. If not set, uses the current font's style.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<font::Style>,
}

pub fn properties() -> Properties {
    Properties::default()
}

impl Properties {
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn style(mut self, style: font::Style) -> Self {
        self.style = Some(style);
        self
    }
}

impl From<()> for Properties {
    fn from(_: ()) -> Self {
        properties()
    }
}

impl From<Color> for Properties {
    fn from(c: Color) -> Self {
        properties().color(c)
    }
}

impl From<font::Style> for Properties {
    fn from(style: font::Style) -> Self {
        properties().style(style)
    }
}

/// Experimental trait for applying a number of properties to another type.
pub trait With<T> {
    fn with(self, other: T) -> Self;
}

impl With<Properties> for Paint {
    fn with(self, text: Properties) -> Self {
        if let Some(color) = text.color {
            return self.color(color);
        }
        self
    }
}

#[test]
pub fn empty_properties_are_serialized_as_an_empty_object() {
    assert_eq!(serde_json::to_string(&properties()).unwrap(), "{}");
}
