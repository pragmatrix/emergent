use crate::{finite, length};

/// A span, a one-dimensional offset and length.
///
/// Note that the offset can be negative.
/// TODO: may a finite scalar type should be introduced.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Span(finite, length);

// Scalar can not be NaN, so implement Eq
impl Eq for Span {}

pub fn span(begin: impl Into<finite>, length: impl Into<length>) -> Span {
    Span(begin.into(), length.into())
}

impl Span {
    pub fn empty() -> Span {
        span(0.0, 0.0)
    }

    pub fn begin(&self) -> finite {
        self.0
    }

    pub fn length(&self) -> length {
        self.1
    }

    pub fn end(&self) -> finite {
        self.begin() + self.length()
    }
}

/// Functions that work over span slices.
pub mod spans {
    use crate::prelude::*;
    use crate::{finite, length, Span};

    /// An iterator that generates all span positions.
    ///
    /// These are all the begin() positions of all spans and the end() position of
    /// the last span.
    pub fn positions<'a>(spans: &'a [Span]) -> impl Iterator<Item = finite> + 'a {
        self::begins(spans).chain(spans.last().into_iter().map(|s| s.end()))
    }

    /// An iterator that returns all span beginnings.
    pub fn begins<'a>(spans: &'a [Span]) -> impl Iterator<Item = finite> + 'a {
        spans.iter().map(|s| s.begin())
    }

    /// An iterator that returns all span endings.
    pub fn ends<'a>(spans: &'a [Span]) -> impl Iterator<Item = finite> + 'a {
        spans.iter().map(|s| s.end())
    }

    /// An iterator that returns all span lengths.
    pub fn lengths<'a>(spans: &'a [Span]) -> impl Iterator<Item = length> + 'a {
        spans.iter().map(|s| s.length())
    }

    /// The span that covers all spans.
    pub fn span(spans: &[Span]) -> Option<Span> {
        spans.first_and_last().map(|(f, l)| {
            let (begin, end) = (f.begin(), l.end());
            let length = length::from(end - begin);
            crate::span(begin, length)
        })
    }
}
