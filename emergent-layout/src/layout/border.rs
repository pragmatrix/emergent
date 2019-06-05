use crate::{constraints, length, Bound, Combine, CompletedAxes, Layout, Linear, ResultRef, Span};

struct Border<'a> {
    inner: &'a mut dyn Layout,
    constraints: [(Linear, Linear); 2],
    result: ResultRef<'a>,
}

impl<'a> Layout for Border<'a> {
    fn compute_constraints(&self, axis: usize) -> Option<Linear> {
        let inner = self.inner.compute_constraints(axis)?;
        Some([self.constraints[axis].0, inner, self.constraints[axis].1].combine_directional())
    }

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
