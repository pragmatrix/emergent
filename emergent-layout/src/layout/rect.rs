use crate::{length, Bound, CompletedAxes, Layout, Linear, Rect, RectHelper, Span};

/// A Rect as a layout.
///
/// The size of the rect acts as the fixed area constraint, and the position is set when positioned.
impl Layout for Rect {
    fn compute_constraints(&self, axis: usize) -> Option<Linear> {
        let length = match axis {
            0 => Some(self.width()),
            1 => Some(self.height()),
            _ => None,
        };

        length.map(|l| Linear::fixed(l.into()))
    }

    fn layout(
        &mut self,
        completed: &CompletedAxes,
        axis: usize,
        bound: Bound,
    ) -> (length, Option<usize>) {
        let length = self.compute_constraints(axis).unwrap().layout(bound);
        self.set_length(axis, length);
        (length, completed.first_incomplete_except(axis))
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
    use crate::{layout_and_position, Bound, Rect};
    #[test]
    fn unbounded_uses_rect_size_as_fixed_constraint() {
        let mut r = Rect::from((10.0, 11.0, 100.0, 110.0));
        layout_and_position(&mut r, &[Bound::Unbounded, Bound::Unbounded], &[1.0, 1.1]);
        assert_eq!(r, Rect::from((1.0, 1.1, 100.0, 110.0)));
    }
}
