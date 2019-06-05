use emergent_drawing::{scalar, DrawingCanvas, Point, Rect, Size};

pub mod constraints;
mod grid;
pub use grid::*;
pub mod layout;
mod primitives;
pub use primitives::*;
mod span;
pub use span::*;

use crate::constraints::Linear;
use crate::layout::Constrain;

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

pub trait LayoutExtensions: Layout {
    fn constrain(&mut self, constraints: &[Linear]) -> Constrain;
}

impl<T: Layout> LayoutExtensions for T {
    fn constrain(&mut self, constraints: &[Linear]) -> Constrain {
        Constrain {
            layout: self,
            constraints: constraints.to_vec(),
        }
    }
}

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

trait RectHelper {
    fn set_pos(&mut self, axis: usize, pos: finite);
    fn set_length(&mut self, axis: usize, length: length);
    fn set_span(&mut self, axis: usize, span: Span) {
        self.set_pos(axis, span.start());
        self.set_length(axis, span.size());
    }
}

impl RectHelper for Rect {
    fn set_pos(&mut self, axis: usize, pos: finite) {
        match axis {
            0 => self.0 = Point(*pos, self.top()),
            1 => self.0 = Point(self.left(), *pos),
            _ => panic!("invalid axis"),
        }
    }

    fn set_length(&mut self, axis: usize, length: length) {
        match axis {
            0 => self.1 = Size(*length, self.height()),
            1 => self.1 = Size(self.width(), *length),
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
use emergent_drawing::{Canvas, Color, Paint, Radius, RoundedRect};
use std::convert::identity;
use std::ops::Range;

#[test]
fn draw_circle() {
    let mut canvas = DrawingCanvas::new();
    let mut paint = &mut Paint::default();
    paint.color = Some(Color(0xff0000f0));
    let rect = Rect::from(((0, 0).into(), (200, 100).into()));
    canvas.draw(RoundedRect::from((rect, Radius(10.0))), &paint);
    canvas.render();
}

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
