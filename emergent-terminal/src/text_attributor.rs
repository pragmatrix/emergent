//! An ansi color formatter.

use crate::ansi;
use crate::index;
use crate::term::color;
use std::io;

#[derive(Copy, Clone, PartialEq, Eq, Default, Debug)]
pub struct Attributes {
    pub color: Option<u8>,
    pub bold: bool,
}

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct AttributedText {
    pub text: String,
    pub attributes: Attributes,
}

/// Runs the input string through the ansi terminal and returns spans of colored text.
pub fn attribute_str(input: &str) -> Vec<AttributedText> {
    if !input.is_ascii() {
        warn!("input is not ascii: {}", &input);
    }
    attribute_bytes(input.as_bytes())
}

pub fn attribute_bytes(bytes: &[u8]) -> Vec<AttributedText> {
    let mut handler = TextAttributeHandler::default();
    let mut processor = ansi::Processor::new();

    let mut dummy_writer = io::Cursor::new(Vec::new());

    for byte in bytes {
        processor.advance(&mut handler, *byte, &mut dummy_writer)
    }

    handler.text
}

#[derive(Default)]
struct TextAttributeHandler {
    pub text: Vec<AttributedText>,
}

impl ansi::Handler for TextAttributeHandler {
    fn input(&mut self, c: char) {
        self.add_char(c);
    }

    fn linefeed(&mut self) {
        self.add_char('\n');
    }

    /// set a terminal attribute
    fn terminal_attribute(&mut self, attr: ansi::Attr) {
        use ansi::Attr::*;

        match attr {
            Bold => self.update_attribute(|a| a.bold = true),
            Reset => self.update_attribute(|a| *a = Attributes::default()),
            Foreground(ansi::Color::Indexed(index)) => {
                self.update_attribute(|a| a.color = Some(index))
            }
            _ => warn!("unsupported attribute change: {:?}", attr),
        }
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

impl TextAttributeHandler {
    fn add_char(&mut self, c: char) {
        self.resolve_current().text.push(c)
    }

    fn update_attribute(&mut self, f: impl FnOnce(&mut Attributes)) {
        let current = self.resolve_current();
        let mut updated_attributes = current.attributes;
        f(&mut updated_attributes);
        if updated_attributes != current.attributes {
            self.new_text().attributes = updated_attributes
        }
    }

    fn resolve_current(&mut self) -> &mut AttributedText {
        if self.text.is_empty() {
            self.text.push(AttributedText::default())
        }
        self.text.last_mut().unwrap()
    }

    pub fn new_text(&mut self) -> &mut AttributedText {
        self.text.push(AttributedText::default());
        self.resolve_current()
    }
}

impl ansi::TermInfo for TextAttributeHandler {
    fn lines(&self) -> index::Line {
        warn!("unexpected line query, returning max");
        index::Line(std::usize::MAX)
    }

    fn cols(&self) -> index::Column {
        warn!("unexpected column query, returning max");
        index::Column(std::usize::MAX)
    }
}
