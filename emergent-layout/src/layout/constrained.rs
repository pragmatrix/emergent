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

#[cfg(test)]
mod test {
    use crate::layout::Constrain;
    use crate::{layout_and_position, Bound, Linear, Point, Rect, Size};
    use emergent_drawing::functions::*;
    use emergent_drawing::{Canvas, DrawingCanvas};

    #[test]
    fn test_contrained_layout() {
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
        let paint = paint().color(0xff0000f0).clone();
        canvas.draw(r, &paint);
        canvas.render();
    }
}
