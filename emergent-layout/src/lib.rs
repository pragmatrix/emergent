use emergent_drawing::{DrawingCanvas, Point, Rect, Size};

pub mod constraints;
pub mod layout;

mod grid;
pub use grid::*;

mod types;
pub use types::*;

pub trait Layout {
    /// Compute the combined constraints of the given axis.
    fn compute_constraints(&self, axis: usize) -> constraints::Dim;

    /// Distribute the available span on the given axis.
    ///
    /// This might change constraints of the other axes.
    fn layout(&mut self, axis: usize, span: Span);
}

pub trait Constrain<'a, L>
where
    L: Layout + 'a,
{
    fn constrain(&'a mut self, constraints: constraints::Rect) -> L;
}

impl<'a> Constrain<'a, layout::Rect<'a>> for Rect {
    fn constrain(&mut self, constraints: constraints::Rect) -> layout::Rect {
        layout::Rect {
            constraints,
            result: self.into(),
        }
    }
}

/// A mut reference to either a Rect or a Grid.
///
/// This type is used as a placeholder that points to the result of a layout
/// computation.
pub enum ResultRef<'a> {
    Rect(&'a mut Rect),
    Grid(&'a mut Grid),
}

impl<'a, 'b> From<&'b mut Rect> for ResultRef<'a>
where
    'b: 'a,
{
    fn from(rect: &'b mut Rect) -> ResultRef<'a> {
        ResultRef::<'a>::Rect(rect)
    }
}

impl<'a> From<&'a mut Grid> for ResultRef<'a> {
    fn from(grid: &mut Grid) -> ResultRef {
        ResultRef::Grid(grid)
    }
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

#[cfg(test)]
use emergent_drawing::{Canvas, Color, Paint, Radius, RoundedRect};

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
    let constraints = [
        constraints::Dim::min(10.into()),
        constraints::Dim::min(20.into()),
    ];
    let mut r = Rect::default();
    let mut layout = r.constrain(constraints);
    layout.layout(0, span(10, 15));

    let mut canvas = DrawingCanvas::new();
    let mut paint = &mut Paint::default();
    paint.color = Some(Color(0xff0000f0));
    canvas.draw(r, &paint);
    canvas.render();
}
