use crate::{constraints, length, Bound, CompletedAxes, Layout, Span};

/// A layout that overrides the constraints of another layout.
pub struct Constrain<'a> {
    pub layout: &'a mut Layout,
    pub constraints: Vec<constraints::Linear>,
}

impl<'a> Layout for Constrain<'a> {
    fn compute_constraints(&self, axis: usize) -> Option<constraints::Linear> {
        self.constraints.get(axis).cloned()
    }

    fn layout(
        &mut self,
        completed: &CompletedAxes,
        axis: usize,
        bound: Bound,
    ) -> (length, Option<usize>) {
        self.layout.layout(completed, axis, bound)
    }

    fn position(&mut self, spans: &[Span]) {
        self.layout.position(spans)
    }
}
