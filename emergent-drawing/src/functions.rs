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

pub fn line_v(x: scalar, (y1, y2): (scalar, scalar)) -> Line {
    line((x, y1), (x, y2))
}

pub fn line_h(y: scalar, (x1, x2): (scalar, scalar)) -> Line {
    line((x1, y), (x2, y))
}

pub fn paint() -> Paint {
    Paint::new()
}
