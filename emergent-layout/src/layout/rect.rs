use crate::{constraints, Layout, ResultRef, Span};

/// Layout a simple rectangular part.
pub struct Rect<'a> {
    pub constraints: constraints::Rect,
    pub result: ResultRef<'a>,
}

impl<'a> Layout for Rect<'a> {
    fn compute_constraints(&self, axis: usize) -> constraints::Dim {
        self.constraints[axis]
    }

    fn layout(&mut self, axis: usize, span: Span) {
        unimplemented!()
        // self.result.set_span(axis, 0, span);
    }
}
