use crate::{finite, length};

/// A span, a one-dimensional offset and length.
///
/// Note that the offset can be negative.
/// TODO: may a finite scalar type should be introduced.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Span(finite, length);

// Scalar can not be NaN, so implement Eq
impl Eq for Span {}

pub fn span(start: impl Into<finite>, len: impl Into<length>) -> Span {
    Span(start.into(), len.into())
}

impl Span {
    pub fn empty() -> Span {
        span(0.0, 0.0)
    }

    pub fn start(&self) -> finite {
        self.0
    }

    pub fn size(&self) -> length {
        self.1
    }
}
