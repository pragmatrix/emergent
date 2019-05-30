use crate::{fps, Span};
use emergent_drawing::Rect;
use std::ops::Range;

/// A type representing a two dimensional orthogonal grid.
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct Grid {
    /// The spans for the two dimensions.
    pub spans: [Vec<Span>; 2],
}

impl Grid {
    pub fn size(&self, axis: usize, range: Range<usize>) -> fps {
        unimplemented!()
    }

    pub fn rect(columns: Range<usize>, rows: Range<usize>) -> Rect {
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
