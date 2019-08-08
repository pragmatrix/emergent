use crate::{Font, Point};
use serde::{Deserialize, Serialize};

/// Text, described by an origin, a string, and the font.
///
/// The origin is treated as the starting point on baseline where the text
/// will be rendered.
// TODO: can we share fonts?
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Text {
    pub origin: Point,
    pub text: String,
    pub font: Font,
}

pub fn text(origin: impl Into<Point>, text: impl AsRef<str>, font: &Font) -> Text {
    Text::new(origin.into(), text.as_ref(), font)
}

impl Text {
    pub fn new(origin: Point, text: &str, font: &Font) -> Self {
        Text {
            origin,
            text: String::from(text),
            font: font.clone(),
        }
    }
}
