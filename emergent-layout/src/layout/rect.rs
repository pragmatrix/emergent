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
    use crate::{
        layout_and_position, Bound, Color, Constrain, DrawingCanvas, Linear, Paint, Point, Rect,
        Size,
    };
    use emergent_drawing::Canvas;

    #[test]
    fn test_simple_rect_layout() {
        let constraints = [Linear::min(10.into()), Linear::min(20.into())];
        let mut r = Rect::default();
        let mut l = r.constrain(&constraints);
        layout_and_position(
            &mut l,
            &[Bound::Bounded(2.into()), Bound::Bounded(4.into())],
            &[1.0, 3.0],
        );

        assert_eq!(r, Rect(Point::from((1, 3)), Size::from((2, 4))));

        let mut canvas = DrawingCanvas::new();
        let mut paint = &mut Paint::default();
        paint.color = Some(Color(0xff0000f0));
        canvas.draw(r, &paint);
        canvas.render();
    }

}
