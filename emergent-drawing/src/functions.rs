use crate::{scalar, Line, Paint, Point, Rect, Size};

pub fn point(x: scalar, y: scalar) -> Point {
    Point(x, y)
}

pub fn size(width: scalar, height: scalar) -> Size {
    Size(width, height)
}

pub fn rect(p: impl Into<Point>, s: impl Into<Size>) -> Rect {
    Rect(p.into(), s.into())
}

pub fn line(p1: impl Into<Point>, p2: impl Into<Point>) -> Line {
    Line(p1.into(), p2.into())
}

pub fn paint() -> Paint {
    Paint::new()
}
