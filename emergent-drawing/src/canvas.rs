//! Usability optimized drawing functions and wrappers.

use crate::drawing_target::DrawingTarget;
use crate::{
    scalar, Arc, Circle, Line, LineSegments, Oval, Paint, Path, Point, Polygon, Radius, Rect,
    RoundedRect, Shape,
};

pub struct Canvas<'a, DT>
where
    DT: DrawingTarget<'a>,
{
    drawing_target: &'a mut DT,
}

impl<'a, DT: DrawingTarget<'a>> Canvas<'a, DT> {
    pub fn from_target(drawing_target: &'a mut DT) -> Self {
        Canvas { drawing_target }
    }

    pub fn draw_circle<IP: Into<Point>, IR: Into<Radius>>(
        &mut self,
        point: IP,
        radius: IR,
        paint: &Paint,
    ) {
        self.draw_shape(Circle(point.into(), radius.into()), paint)
    }

    // TODO: should this be pub?
    fn draw_shape<IS: Into<Shape>>(&mut self, shape: IS, paint: &Paint) {
        self.drawing_target.draw(shape.into(), paint);
    }
}

//
// From scalar conversions
//

impl From<(scalar, scalar)> for Point {
    fn from((x, y): (f64, f64)) -> Self {
        Point(x, y)
    }
}

impl From<scalar> for Radius {
    fn from(radius: scalar) -> Self {
        Radius(radius)
    }
}

//
// From i64 conversion
//

impl From<(i64, i64)> for Point {
    fn from((x, y): (i64, i64)) -> Self {
        (x as scalar, y as scalar).into()
    }
}

impl From<i64> for Radius {
    fn from(radius: i64) -> Self {
        (radius as scalar).into()
    }
}

//
// Shape From implementations.
//

impl From<Point> for Shape {
    fn from(point: Point) -> Self {
        Shape::Point(point)
    }
}

impl From<Line> for Shape {
    fn from(line: Line) -> Self {
        Shape::Line(line)
    }
}

impl From<LineSegments> for Shape {
    fn from(line_segments: LineSegments) -> Self {
        Shape::LineSegments(line_segments)
    }
}

impl From<Polygon> for Shape {
    fn from(polygon: Polygon) -> Self {
        Shape::Polygon(polygon)
    }
}

impl From<Rect> for Shape {
    fn from(rect: Rect) -> Self {
        Shape::Rect(rect)
    }
}

impl From<Oval> for Shape {
    fn from(oval: Oval) -> Self {
        Shape::Oval(oval)
    }
}

impl From<RoundedRect> for Shape {
    fn from(rounded_rect: RoundedRect) -> Self {
        Shape::RoundedRect(rounded_rect)
    }
}

impl From<Circle> for Shape {
    fn from(circle: Circle) -> Self {
        Shape::Circle(circle)
    }
}

impl From<Arc> for Shape {
    fn from(arc: Arc) -> Self {
        Shape::Arc(arc)
    }
}

impl From<Path> for Shape {
    fn from(path: Path) -> Self {
        Shape::Path(path)
    }
}