/*
use crate::Layout;

pub struct Grid<'a> {
    pub layouts: &'a [&'a [&'a dyn Layout]],
}

impl<'a> Layout for Grid<'a> {
    fn compute_constraints(&self, axis: usize) -> Option<Linear> {}

    fn layout(
        &mut self,
        completed_axes: &CompletedAxes,
        axis: usize,
        bound: Bound,
    ) -> (length, Option<usize>) {
        unimplemented!()
    }

    fn position(&mut self, spans: &[Span]) {
        unimplemented!()
    }
}
*/
