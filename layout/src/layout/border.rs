use crate::constraints::{Combine, Linear};
use crate::{length, ConstraintContext, Layout, LayoutContext, ResultRef, Span};

pub struct Border<'a> {
    inner: &'a mut dyn Layout,
    constraints: [(Linear, Linear); 2],
    _result: ResultRef<'a>,
}

impl<'a> Layout for Border<'a> {
    fn compute_constraints(&self, context: &ConstraintContext) -> Option<Linear> {
        let inner = self.inner.compute_constraints(context)?;
        let axis = context.axis;
        Some([self.constraints[axis].0, inner, self.constraints[axis].1].combine_directional())
    }

    fn layout(&mut self, _context: &LayoutContext) -> (length, Option<usize>) {
        unimplemented!()
    }

    fn position(&mut self, _spans: &[Span]) {
        unimplemented!()
    }
}
