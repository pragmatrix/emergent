use crate::{Font, Point};
use serde::{Deserialize, Serialize};

/// Text, described by a location, a string, and the font.
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
            origin: origin,
            text: String::from(text),
            font: font.clone(),
        }
    }
}
