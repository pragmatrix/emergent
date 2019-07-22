use crate::constraints::{Combine, Linear};
use emergent_drawing::{scalar, Point, Rect, Vector};
use std::convert::identity;

pub mod constraints;

mod grid;
pub use grid::*;

pub mod layout;

mod prelude;

mod primitives;
pub use primitives::*;

mod span;
pub use span::*;

/// A ResultRef is just a mutable Rectangle for now.
pub type ResultRef<'a> = &'a mut Rect;

pub trait Layout {
    /// Compute the constraints of the given axis.
    ///
    /// Returns None if layout of this axis is not supported.
    fn compute_constraints(&self, axis: usize) -> Option<Linear>;

    /// Layouts the given axis according to the given bound.
    ///
    /// The element is supposed to return a finite positive length of the current's
    /// axis size, and the next axis to layout, or None if layout of
    /// all axes is completed. The returned axis is not allowed to contain an axis
    /// that has already been completed layout.
    fn layout(
        &mut self,
        completed_axes: &CompletedAxes,
        axis: usize,
        bound: Bound,
    ) -> (length, Option<usize>);

    /// Positions this layout on all the axes.
    fn position(&mut self, spans: &[Span]);
}

/// A layout bound on an axis.
///
/// TOOD: this looks similar to Max.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Bound {
    Unbounded,
    Bounded(length),
}

trait RectHelper {
    fn set_begin(&mut self, axis: usize, pos: finite);
    fn set_length(&mut self, axis: usize, length: length);
    fn set_span(&mut self, axis: usize, span: Span) {
        self.set_begin(axis, span.begin());
        self.set_length(axis, span.length());
    }
}

impl RectHelper for Rect {
    fn set_begin(&mut self, axis: usize, pos: finite) {
        match axis {
            0 => self.0 = Point::new(*pos, self.top()),
            1 => self.0 = Point::new(self.left(), *pos),
            _ => panic!("invalid axis"),
        }
    }

    fn set_length(&mut self, axis: usize, length: length) {
        match axis {
            0 => self.1.x = self.0.x + *length,
            1 => self.1.y = self.0.y + *length,
            _ => {}
        }
    }
}

pub fn layout<L>(layout: &mut L, bounds: &[Bound]) -> Vec<length>
where
    L: Layout,
{
    let axes = bounds.len();
    let (bounded, unbounded): (Vec<usize>, Vec<usize>) =
        (0..axes).partition(|axis| bounds[*axis] != Bound::Unbounded);

    // if layout() does not return a new recommended axis, this is the default
    // order of the axes to be layouted, first all bounded then the unbounded
    // ones.
    let ordered: Vec<usize> = vec![bounded, unbounded].into_iter().flatten().collect();

    let mut axis = *ordered.first().unwrap();
    let mut completed = CompletedAxes::new(axes);
    let mut lengths: Vec<length> = vec![0.0.into(); axes];

    loop {
        let (length, next_axis) = layout.layout(&completed, axis, bounds[axis]);
        lengths[axis] = length;
        completed.complete_axis(axis);
        if completed.is_complete() {
            assert_eq!(next_axis, None);
            break;
        }

        axis = match next_axis {
            Some(axis) => {
                assert!(!completed.is_axis_complete(axis));
                axis
            }
            None => *ordered
                .iter()
                .find(|axis| !completed.is_axis_complete(**axis))
                .unwrap(),
        }
    }

    lengths
}

pub fn layout_and_position<L>(layout: &mut L, bounds: &[Bound], positions: &[scalar])
where
    L: Layout,
{
    let lengths = self::layout(layout, bounds);
    let spans: Vec<Span> = positions
        .iter()
        .enumerate()
        .map(|(i, p)| span(*p, lengths[i]))
        .collect();
    layout.position(&spans);
}

/*

/// Excess distributor preferences.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
struct ExcessDistribution {
    /// The priority of this element for excess distribution.
    ///
    /// Elements of the highest priority on an axis receive excess space
    /// first, then the rest goes to the ones below.
    /// Default is 1, Priority 0 is special, here the element may be
    /// removed completely if there is not enough space and now wrapping possible.
    priority: usize,
    /// The weight relative to other's on the same axis and same priority.
    /// This is used when excess dims are distributed for a priority group.
    weight: fps,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct DimProperties {
    constraints: DimConstraints,
    distribution: ExcessDistribution,
}

*/

/// Flags for each of the axes currently in layout that are completed.
///
/// TODO: use BitVec for that (crate bit_vec).
#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct CompletedAxes(Vec<bool>);

impl CompletedAxes {
    pub fn new(axes: usize) -> CompletedAxes {
        CompletedAxes(vec![false; axes])
    }

    pub fn is_complete(&self) -> bool {
        self.0.iter().cloned().all(identity)
    }

    pub fn is_axis_complete(&self, axis: usize) -> bool {
        self.0[axis]
    }

    pub fn complete_axis(&mut self, axis: usize) -> &mut Self {
        self.0[axis] = true;
        self
    }

    /// The first incomplete axis.
    pub fn first_incomplete(&self) -> Option<usize> {
        self.0.iter().cloned().position(|b| !b)
    }

    pub fn first_incomplete_except(&self, axis: usize) -> Option<usize> {
        self.clone().complete_axis(axis).first_incomplete()
    }
}

#[cfg(test)]
mod tests {
    use emergent_drawing::functions::*;
    use emergent_drawing::{Canvas, DrawingCanvas, Radius, RoundedRect};

    #[test]
    fn draw_circle() {
        let mut canvas = DrawingCanvas::new();
        let paint = paint().color(0xff0000f0).clone();
        let rect = rect((0, 0), (200, 100));
        canvas.draw(RoundedRect::from((rect, Radius::new(10.0))), &paint);
        canvas.render();
    }
}
