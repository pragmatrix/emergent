use crate::{constraints, fps, Layout, ResultRef, Span};

/// Layout of text.
struct Text<'a> {
    /// The axis in which the text flows. also the writing direction.
    flow_axis: usize,
    /// The axis in which the text overflows when there is not enough space left.
    overflow_axis: usize,
    /// The minimum size of the flow axis, default 0.
    minimum_flow_size: fps,
    /// The minimum size of the overflow axis.
    ///
    /// Recommended to set this to the
    /// line height.
    minimum_overflow_size: fps,
    /// The function to compute the size of the overflow axis from a given
    /// size in the writing direction (the flow axis).
    compute_overflow: &'a dyn Fn(fps) -> fps,

    // computed values for overflow_size. These value is computed
    // as soon the flow_axis is layouted.
    overflow_size_computed: Option<fps>,

    /// The resulting layouted Grid.
    pub result: ResultRef<'a>,
}

impl<'a> Layout for Text<'a> {
    fn compute_constraints(&self, axis: usize) -> constraints::Dim {
        if axis == self.flow_axis {
            constraints::Dim::min(self.minimum_flow_size)
        } else if axis == self.overflow_axis {
            constraints::Dim::min(
                self.minimum_overflow_size
                    .max(self.overflow_size_computed.unwrap_or(fps::ZERO)),
            )
        } else {
            panic!("unsupported text axis: {}", axis);
        }
    }

    fn layout(&mut self, axis: usize, span: Span) {
        if axis == self.flow_axis {
            self.overflow_size_computed = Some((self.compute_overflow)(span.size()))
        }

        // self.result.set_span(axis, 0, span)
    }
}
