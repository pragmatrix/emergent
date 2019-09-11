use serde::{Deserialize, Serialize};

pub(crate) mod arc;
pub use arc::*;

pub(crate) mod circle;
pub use circle::*;

pub(crate) mod image;
pub use image::*;

pub(crate) mod line;
pub use line::*;

pub(crate) mod oval;
pub use oval::*;

pub(crate) mod path;
pub use path::Path;

pub(crate) mod point;
pub use point::*;

pub(crate) mod polygon;
pub use polygon::*;

pub(crate) mod rect;
pub use rect::*;

pub(crate) mod rounded_rect;
pub use rounded_rect::*;

pub mod text;
pub use text::Text;

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Shape {
    Point(Point),
    Line(Line),
    Polygon(Polygon),
    Rect(Rect),
    Oval(Oval),
    RoundedRect(RoundedRect),
    // TODO: Skia has an optimized function for drawing a rounded rect inside another. Should we support that?
    Circle(Circle),
    Arc(Arc),
    Path(Path),
    Image(ImageId, Option<Rect>, Rect),
    // ImageNine?
    Text(Text),
}

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

pub trait IntoShape {
    fn into_shape(self) -> Shape;
}

impl<T: Into<Shape>> IntoShape for T {
    fn into_shape(self) -> Shape {
        self.into()
    }
}
