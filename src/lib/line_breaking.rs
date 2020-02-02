//! Line breaking algorithms.

use emergent_drawing::{scalar, Point, Vector};
use std::iter;

/// Breaks a string into lines by splitting at \n characters.
///
/// The implementation is symmetric in the sense that the returned lines
/// concatenated with `\n` will result the same string.
///
/// This also means, that the `\r` character will _always_ be treated as
/// whitespace and that no unicode line breaks are supported.
pub fn text_as_lines(text: &str) -> impl Iterator<Item = &str> {
    let mut next = Some(0);

    let f = move || {
        if let Some(current) = next {
            match text[current..].find('\n') {
                Some(len) => {
                    let end = current + len;
                    let r = &text[current..end];
                    next = Some(end + 1);
                    Some(r)
                }
                None => {
                    let r = Some(&text[current..text.len()]);
                    next = None;
                    r
                }
            }
        } else {
            None
        }
    };

    iter::from_fn(f)
}

/// Representation of a text origin position.
///
/// Useful when drawing lines.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct TextOrigin {
    /// Origin of the textbox.
    origin: Point,

    /// The offset from the origin to the point to draw.
    advance: Vector,
}

impl TextOrigin {
    pub fn new(origin: Point) -> TextOrigin {
        TextOrigin {
            origin,
            advance: Vector::default(),
        }
    }

    pub fn point(&self) -> Point {
        self.origin + self.advance
    }

    /// Add a newline: reset horizontal advance and move one line down.
    pub fn newline(&mut self, line_spacing: scalar) {
        self.advance = Vector::new(0.0, self.advance.y + line_spacing)
    }

    /// Add horizontal advance.
    pub fn advance(&mut self, advance: scalar) {
        self.advance.x += advance
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn single() {
        assert_eq!(tal("single"), vec!["single"])
    }

    #[test]
    fn empty() {
        assert_eq!(tal(""), vec![""])
    }

    #[test]
    fn two() {
        assert_eq!(tal("one\ntwo"), vec!["one", "two"])
    }

    #[test]
    fn two_nl() {
        assert_eq!(tal("one\ntwo\n"), vec!["one", "two", ""])
    }

    #[test]
    fn nl_one_nl() {
        assert_eq!(tal("\none\n"), vec!["", "one", ""])
    }

    fn tal(text: &str) -> Vec<&str> {
        super::text_as_lines(text).collect()
    }
}
