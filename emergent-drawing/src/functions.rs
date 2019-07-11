use crate::font::Style;
use crate::{font, scalar, Extent, Font, Line, Paint, Point, Rect, Text, Vector};

pub fn point(x: scalar, y: scalar) -> Point {
    Point(x, y)
}

pub fn extent(width: scalar, height: scalar) -> Extent {
    Extent::from((width, height))
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

pub fn rect(p: impl Into<Point>, e: impl Into<Vector>) -> Rect {
    Rect(p.into(), e.into())
}

pub fn font(typeface_name: impl AsRef<str>, size: scalar) -> Font {
    Font(
        typeface_name.as_ref().to_owned(),
        Style::NORMAL,
        font::Size(size),
    )
}

pub fn text(p: impl Into<Point>, text: impl AsRef<str>, font: &Font) -> Text {
    Text(p.into(), text.as_ref().to_owned(), font.clone())
}

pub fn paint() -> Paint {
    Paint::new()
}
