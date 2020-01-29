use crate::constraints::Linear;
use crate::{constraints, length, ConstraintContext, Layout, LayoutContext, Span};

/// A layout that overrides the constraints of another layout.
pub struct Constrained<'a> {
    pub layout: &'a mut dyn Layout,
    pub constraints: Vec<constraints::Linear>,
}

impl<'a> Layout for Constrained<'a> {
    fn compute_constraints(&self, context: &ConstraintContext) -> Option<constraints::Linear> {
        self.constraints.get(context.axis).cloned()
    }

    fn layout(&mut self, context: &LayoutContext) -> (length, Option<usize>) {
        self.layout.layout(context)
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
    use crate::constraints::Linear;
    use crate::{layout_and_position, Bound, Constrain, UnimplementedMeasure};
    use emergent_drawing::functions::*;
    use emergent_drawing::{paint::paint, Drawing, DrawingTarget, Rect, Render};

    #[test]
    fn test_contrained_layout() {
        let constraints = [Linear::min(10.into()), Linear::min(20.into())];
        let mut r = Rect::default();
        let mut l = r.constrain(&constraints);
        layout_and_position(
            &mut l,
            UnimplementedMeasure,
            &[Bound::Bounded(2.into()), Bound::Bounded(4.into())],
            &[1.0, 3.0],
        );

        assert_eq!(
            r,
            Rect::new(point(1.0, 3.0), point(1.0, 3.0) + vector(2.0, 4.0))
        );

        let mut canvas = Drawing::new();
        let paint = paint().color(0xff0000f0);
        canvas.draw(r, paint);
        canvas.render();
    }
}
