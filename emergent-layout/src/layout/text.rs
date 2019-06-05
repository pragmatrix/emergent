use crate::constraints::Linear;
use crate::{constraints, fps, Bound, CompletedAxes, Layout, RectHelper, ResultRef, Span};
use std::cell::Cell;
use std::hash::Hash;

pub trait MeasureText {
    type Text;

    /// Returns the length of the text in the writing direction and the length in
    /// the line layout direction.
    ///
    /// If the maximum length is specified, glyphs _should_ not exceed the maximum
    /// length in the writing direction.
    ///
    /// If the maximum length is None, the text should be layouted without
    /// imposed wrapping.
    fn measure_text(&self, text: &Self::Text, max: Option<fps>) -> (fps, fps);
}

/// Layout of text.
struct Text<'a, MT: MeasureText> {
    /// The axis in which the text flows. also the writing direction.
    main_axis: usize,

    /// The axis in which the text overflows when there is not enough space left.
    cross_axis: usize,

    text: &'a MT::Text,

    /// The facility that is able to measure text.
    measure_text: &'a MT,

    /// Computes the constraints of the main and the cross axis for a text that
    /// has no bounds.  
    ///
    /// This function is never called when the the main axis's length is bounded.
    ///
    /// The default returns fixed linear constraints that are set to the size of the text in
    /// the writing direction without imposed wrapping.
    constraints: Option<&'a dyn Fn(&MT, &MT::Text) -> (Linear, Linear)>,

    /// The result.
    pub result: ResultRef<'a>,
}

impl<'a, MT: MeasureText> Layout for Text<'a, MT> {
    fn compute_constraints(&self, axis: usize) -> Option<constraints::Linear> {
        let (main, cross) = self.resolve_constraints();
        if axis == self.main_axis {
            Some(main)
        } else if axis == self.cross_axis {
            Some(cross)
        } else {
            None
        }
    }

    fn layout(
        &mut self,
        completed: &CompletedAxes,
        axis: usize,
        bound: Bound,
    ) -> (fps, Option<usize>) {
        if axis == self.main_axis {
            match bound {
                Bound::Unbounded => {
                    let (length, _) = self.measure_text.measure_text(self.text, None);
                    (length, completed.first_incomplete_except(axis))
                }
                Bound::Bounded(length) => {
                    let (length, _) = self.measure_text.measure_text(self.text, Some(length));
                    (length, completed.first_incomplete_except(axis))
                }
            }
        } else if axis == self.cross_axis {
            // for the cross-axis it's too late to bound it, so do the best.
            let (_, length) = self.measure_text.measure_text(self.text, None);
            (length, completed.first_incomplete_except(axis))
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

impl<'a, MT: MeasureText> Text<'a, MT> {
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
    }
}
