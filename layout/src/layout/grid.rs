use crate::{length, Span};
use emergent_drawing::Rect;
use std::ops::Range;

/// A type representing a two dimensional orthogonal grid.
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct Grid {
    /// The spans for the two dimensions.
    pub spans: [Vec<Span>; 2],
}

impl Grid {
    pub fn size(&self, _axis: usize, _range: Range<usize>) -> length {
        unimplemented!()
    }

    pub fn rect(_columns: Range<usize>, _rows: Range<usize>) -> Rect {
        unimplemented!()
    }

    pub fn set_span(&mut self, axis: usize, index: usize, span: Span) {
        let spans = &mut self.spans[axis];
        if index >= spans.len() {
            spans.resize(index + 1, Span::empty())
        }
        spans[index] = span
    }
}
