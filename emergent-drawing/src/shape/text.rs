use crate::{Color, Drawing, Font, Paint, Point};
use serde::{Deserialize, Serialize};
use serde_tuple::*;
use std::ops::Range;

/// Text, described by a, the font, and an origin.
///
/// The origin is treated as the starting point on baseline where the text
/// will be rendered.
// TODO: can we share fonts?
#[derive(Clone, Serialize_tuple, Deserialize_tuple, PartialEq, Debug)]
pub struct Text {
    pub text: String,
    pub font: Font,
    pub origin: Point,
    pub runs: Vec<Run>,
}

pub fn text(text: impl AsRef<str>, font: &Font, origin: impl Into<Option<Point>>) -> Text {
    Text::new(text.as_ref(), font, origin.into().unwrap_or_default())
}

impl Text {
    pub fn new(text: &str, font: &Font, origin: Point) -> Self {
        Text {
            text: String::from(text),
            font: font.clone(),
            origin,
            runs: Vec::new(),
        }
    }
}

/// Text runs describing that describe formatted text.
///
/// Inspired by WPF.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Run {
    /// Draw Text with the given properties.
    Text(Range<usize>, Properties),
    /// A newline.
    EndOfLine,

    /// Another text block. This can be used to render text on the same baseline
    /// with other fonts.
    Block(Text),
    /// Drawing layouted on the text line at baseline + point.
    ///
    /// Size is defined by fast_bounds().
    Drawing(Drawing, Point),
}

/// Creates a new block of text.
pub fn block(font: &Font, origin: impl Into<Option<Point>>) -> Text {
    Text {
        text: String::new(),
        runs: Vec::new(),
        font: font.clone(),
        origin: origin.into().unwrap_or_default(),
    }
}

impl Text {
    /// Add a text run with the given paint.
    pub fn text(&mut self, text: impl AsRef<str>, properties: impl Into<Properties>) -> &mut Self {
        let text = text.as_ref();
        let start = self.text.len();
        self.text.push_str(text);
        self.runs
            .push(Run::Text(start..self.text.len(), properties.into()));
        self
    }

    /// Indicate the end of the current line.
    pub fn eol(&mut self) -> &mut Self {
        self.runs.push(Run::EndOfLine);
        self
    }
}

/// Properties for that descripe how the text should be rendered.
///
/// Value type semantics.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct Properties {
    /// A custom color for text. If not set, uses the color of the current Paint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
}

pub fn properties() -> Properties {
    Properties::default()
}

impl Properties {
    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }
}

impl From<Color> for Properties {
    fn from(c: Color) -> Self {
        properties().color(c)
    }
}

impl From<()> for Properties {
    fn from(_: ()) -> Self {
        properties()
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
