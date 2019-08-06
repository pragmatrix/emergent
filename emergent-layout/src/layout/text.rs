use crate::{
    constraints, layout::RectHelper, length, Bound, ConstraintContext, Layout, LayoutContext,
    ResultRef, Span,
};
use emergent_drawing::Font;

/// Layout of text.
pub struct Text<'a> {
    font: &'a Font,
    text: &'a str,

    layout_axis: usize,
    cross_axis: usize,

    /// The result.
    result: ResultRef<'a>,
}

impl<'a> Layout for Text<'a> {
    fn compute_constraints(&self, context: &ConstraintContext) -> Option<constraints::Linear> {
        let _axis = context.axis;
        unimplemented!();
        /*
        let (main, cross) = self.resolve_constraints();
        if axis == self.layout_axis {
            Some(main)
        } else if axis == self.cross_axis {
            Some(cross)
        } else {
            None
        } */
    }

    fn layout(&mut self, context: &LayoutContext) -> (length, Option<usize>) {
        let axis = context.axis;
        if axis == self.layout_axis {
            match context.bound {
                Bound::Unbounded => {
                    let (length, _) = context.measure.text(self.text, self.font, None);
                    (length, context.completed.first_incomplete_except(axis))
                }
                Bound::Bounded(length) => {
                    let (length, _) = context.measure.text(self.text, self.font, Some(length));
                    (length, context.completed.first_incomplete_except(axis))
                }
            }
        } else if axis == self.cross_axis {
            // for the cross-axis it's too late to bound it, so do the best.
            let (_, length) = context.measure.text(self.text, self.font, None);
            (length, context.completed.first_incomplete_except(axis))
        } else {
            panic!("unsupported text axis")
        }
    }

    fn position(&mut self, spans: &[Span]) {
        spans
            .iter()
            .enumerate()
            .for_each(|(i, span)| self.result.set_span(i, *span));
    }
}

impl<'a> Text<'a> {
    /*
    fn resolve_constraints(&self) -> (Linear, Linear) {
        match self.constraints {
            Some(c) =>
            // TODO: cache the result!
            {
                c(self.measure_text, self.text)
            }
            None => {
                let (main, cross) = self.measure_text.measure_text(self.text, None);
                (Linear::fixed(main), Linear::fixed(cross))
            }
        }
    }*/
}

#[test]
fn text_size() {}
