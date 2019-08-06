use crate::constraints::Linear;
use crate::layout::RectHelper;
use crate::{length, ConstraintContext, Layout, LayoutContext, Span};
use emergent_drawing::Rect;

/// A Rect as a layout.
///
/// The size of the rect acts as the fixed area constraint, and the position is set when positioned.
impl Layout for Rect {
    fn compute_constraints(&self, context: &ConstraintContext) -> Option<Linear> {
        let length = match context.axis {
            0 => Some(self.width()),
            1 => Some(self.height()),
            _ => None,
        };

        length.map(|l| Linear::fixed(l.into()))
    }

    fn layout(&mut self, context: &LayoutContext) -> (length, Option<usize>) {
        let (axis, bound) = (context.axis, context.bound);
        let length = self
            .compute_constraints(&context.constraint())
            .unwrap()
            .layout(bound);
        self.set_length(axis, length);
        (length, context.completed.first_incomplete_except(axis))
    }

    fn position(&mut self, spans: &[Span]) {
        spans
            .iter()
            .enumerate()
            .for_each(|(axis, span)| self.set_span(axis, *span))
    }
}

#[cfg(test)]
mod tests {
    use crate::{layout_and_position, Bound};
    use emergent_drawing::Rect;
    #[test]
    fn unbounded_uses_rect_size_as_fixed_constraint() {
        let mut r = Rect::from(((10.0, 11.0).into(), (90.0, 99.0).into()));
        layout_and_position(&mut r, &[Bound::Unbounded, Bound::Unbounded], &[1.0, 1.1]);
        assert_eq!(r, Rect::from(((1.0, 1.1).into(), (99.0, 109.9).into())));
    }
}
