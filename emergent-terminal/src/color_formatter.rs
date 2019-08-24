//! An ansi color formatter.

use crate::ansi;
use crate::index;
use crate::term::color;
use std::io;

pub struct ColoredText {
    pub text: String,
    pub color: Option<color::Rgb>,
}

/// Runs the input string through the ansi terminal and returns spans of colored text.
pub fn format_str(input: &str) -> Vec<ColoredText> {
    if !input.is_ascii() {
        warn!("input is not ascii: {}", &input);
    }
    format_bytes(input.as_bytes())
}

pub fn format_bytes(bytes: &[u8]) -> Vec<ColoredText> {
    let mut handler = ColoredTextHandler::default();
    let mut processor = ansi::Processor::new();

    let mut dummy_writer = io::Cursor::new(Vec::new());

    for byte in bytes {
        processor.advance(&mut handler, *byte, &mut dummy_writer)
    }

    handler.text
}

#[derive(Default)]
struct ColoredTextHandler {
    pub text: Vec<ColoredText>,
}

impl ansi::Handler for ColoredTextHandler {
    fn input(&mut self, c: char) {
        self.add_char(c);
    }

    fn linefeed(&mut self) {
        self.add_char('\n');
    }

    fn set_color(&mut self, index: usize, _: color::Rgb) {
        warn!("set_color: unsupported, index: {}", index);
    }

    fn dynamic_color_sequence<W: io::Write>(&mut self, _: &mut W, _: u8, _: usize) {
        warn!("dynamic_color_sequence: unsupported");
    }

    fn reset_color(&mut self, index: usize) {
        warn!("reset_color: unsupported, index: {}", index);
    }
}

impl ColoredTextHandler {
    fn add_char(&mut self, c: char) {
        if self.text.is_empty() {
            self.text.push(ColoredText {
                text: String::new(),
                color: None,
            })
        }
        self.text.last_mut().unwrap().text.push(c);
    }
}

impl ansi::TermInfo for ColoredTextHandler {
    fn lines(&self) -> index::Line {
        warn!("unexpected line query, returning max");
        index::Line(std::usize::MAX)
    }

    fn cols(&self) -> index::Column {
        warn!("unexpected column query, returning max");
        index::Column(std::usize::MAX)
    }
}
