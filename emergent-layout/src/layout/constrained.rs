use crate::{constraints, length, Bound, CompletedAxes, Layout, Linear, Span};

/// A layout that overrides the constraints of another layout.
pub struct Constrained<'a> {
    pub layout: &'a mut Layout,
    pub constraints: Vec<constraints::Linear>,
}

impl<'a> Layout for Constrained<'a> {
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

pub trait Constrain: Layout {
    fn constrain(&mut self, constraints: &[Linear]) -> Constrained;
}

impl<T> Constrain for T
where
    T: Layout,
{
    fn constrain(&mut self, constraints: &[Linear]) -> Constrained {
        Constrained {
            layout: self,
            constraints: constraints.to_vec(),
        }
    }
}
