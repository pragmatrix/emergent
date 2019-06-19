use crate::{
    scalar, Arc, Circle, Line, Oval, Path, Point, Polygon, Radius, Rect, RoundedRect, Shape, Text,
};

//
// Shape
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

impl From<Text> for Shape {
    fn from(text: Text) -> Self {
        Shape::Text(text)
    }
}

//
// Line
//

impl From<(Point, Point)> for Line {
    fn from((p1, p2): (Point, Point)) -> Self {
        Line(p1, p2)
    }
}

impl From<(scalar, scalar, scalar, scalar)> for Line {
    fn from((p1x, p1y, p2x, p2y): (scalar, scalar, scalar, scalar)) -> Self {
        Line::from(((p1x, p1y).into(), (p2x, p2y).into()))
    }
}

//
// Radius
//

impl From<scalar> for Radius {
    fn from(radius: scalar) -> Self {
        Radius(radius)
    }
}

impl From<i64> for Radius {
    fn from(radius: i64) -> Self {
        (radius as scalar).into()
    }
}
