use crate::{
    Arc, Bounds, Circle, Clip, Clipped, Contains, Drawing, Extent, Line, Outset, Oval, Point,
    Polygon, Rect, RoundedRect, Shape, Text, Transform, Transformed, Union,
};

pub trait MeasureText {
    /// Measure the given text bounds.
    ///
    /// The returned bounds are returned so that position 0,0 is the text's baseline starting point.
    fn measure_text(&self, text: &Text) -> Bounds;
}

/// Joinable bounding rectangle that support the explicit states empty and unbounded.
///
/// `DrawingBounds::Empty` denotes empty bounds.
/// `DrawingBounds::Unbounded` represents an infinite bound.
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum DrawingBounds {
    /// Nothing to be drawn.
    Empty,
    /// May fill everything given to it.
    Unbounded,
    /// Bounded.
    Bounded(Bounds),
}

impl From<Bounds> for DrawingBounds {
    fn from(bounds: Bounds) -> Self {
        DrawingBounds::Bounded(bounds)
    }
}

impl DrawingBounds {
    pub fn as_bounds(&self) -> Option<&Bounds> {
        match self {
            DrawingBounds::Empty => None,
            DrawingBounds::Unbounded => None,
            DrawingBounds::Bounded(b) => Some(b),
        }
    }
}

impl Clipped for DrawingBounds {
    fn clipped(self, clip: impl Into<Clip>) -> Self {
        let clip = clip.into().fast_bounds().into();
        Self::intersect(&clip, &self)
    }
}

impl Transformed for DrawingBounds {
    fn transformed(self, transform: impl Into<Transform>) -> Self {
        let transform = transform.into();
        self.map_bounded(|b| transform.to_matrix().map_bounds(*b))
    }
}

impl Contains<Point> for DrawingBounds {
    fn contains(&self, point: Point) -> bool {
        match self {
            DrawingBounds::Empty => false,
            DrawingBounds::Unbounded => true,
            DrawingBounds::Bounded(bounds) => bounds.contains(point),
        }
    }
}

pub trait FastBounds {
    fn fast_bounds(&self) -> Bounds;
}

pub trait ComplexFastBounds {
    fn fast_bounds(&self, measure: &dyn MeasureText) -> Bounds;
}

pub trait DrawingFastBounds {
    fn fast_bounds(&self, measure: &dyn MeasureText) -> DrawingBounds;
}

//
// FastBounds
//

impl FastBounds for Point {
    fn fast_bounds(&self) -> Bounds {
        Bounds::from((*self, Extent::EMPTY))
    }
}

impl FastBounds for Line {
    fn fast_bounds(&self) -> Bounds {
        Bounds::from_points(&self.points()).unwrap()
    }
}

impl FastBounds for Rect {
    fn fast_bounds(&self) -> Bounds {
        self.bounds()
    }
}

impl FastBounds for Polygon {
    fn fast_bounds(&self) -> Bounds {
        Bounds::from_points(self.points()).unwrap()
    }
}

impl FastBounds for Oval {
    fn fast_bounds(&self) -> Bounds {
        self.rect().fast_bounds()
    }
}

impl FastBounds for Circle {
    fn fast_bounds(&self) -> Bounds {
        self.to_oval().fast_bounds()
    }
}

impl FastBounds for RoundedRect {
    fn fast_bounds(&self) -> Bounds {
        self.rect().fast_bounds()
    }
}

impl FastBounds for Arc {
    fn fast_bounds(&self) -> Bounds {
        // TODO: find out if a conversion to a list of conics is fast enough for
        // considering it a fast_bounds() computation.
        // Note that a Path's fast_bounds() is in consistent in that it
        // will be more precise, because adding an arc to a Path will be converted to
        // a list of conics.
        self.oval.fast_bounds()
    }
}

impl FastBounds for Clip {
    fn fast_bounds(&self) -> Bounds {
        match self {
            Clip::Rect(r) => r.fast_bounds(),
            Clip::RoundedRect(rr) => rr.fast_bounds(),
            Clip::Path(p) => p.fast_bounds(),
        }
    }
}

//
// ComplexFastBounds
//

impl ComplexFastBounds for Shape {
    fn fast_bounds(&self, measure_text: &dyn MeasureText) -> Bounds {
        match self {
            Shape::Point(p) => p.fast_bounds(),
            Shape::Line(l) => l.fast_bounds(),
            Shape::Polygon(p) => p.fast_bounds(),
            Shape::Rect(r) => r.fast_bounds(),
            Shape::Oval(o) => o.fast_bounds(),
            Shape::RoundedRect(rr) => rr.fast_bounds(),
            Shape::Circle(c) => c.fast_bounds(),
            Shape::Arc(a) => a.fast_bounds(),
            // TODO: review Path here (do we need to support empty paths here)
            Shape::Path(p) => p.fast_bounds(),
            Shape::Image(_, _, target) => target.fast_bounds(),
            // TODO: handle empty text?
            Shape::Text(text) => measure_text.measure_text(&text),
        }
    }
}

impl DrawingFastBounds for Drawing {
    fn fast_bounds(&self, measure: &dyn MeasureText) -> DrawingBounds {
        use Drawing::*;
        match self {
            Empty => DrawingBounds::Empty,
            WithPaint(_, drawing) => drawing.fast_bounds(measure),
            Transformed(transform, drawing) => {
                let nested_bounds = drawing.fast_bounds(measure);
                nested_bounds.map_bounded(|b| transform.to_matrix().map_bounds(*b))
            }
            Clipped(clip, drawing) => {
                let clip = DrawingBounds::Bounded(clip.fast_bounds());
                let drawing = drawing.fast_bounds(measure);
                DrawingBounds::intersect(&clip, &drawing)
            }
            BackToFront(drawings) => {
                DrawingBounds::union_all(drawings.iter().map(|d| d.fast_bounds(measure)))
            }
            Fill(_) => DrawingBounds::Unbounded,
            Shape(shape) => DrawingBounds::Bounded(shape.fast_bounds(measure)),
        }
    }
}

impl DrawingBounds {
    pub fn map_bounded(&self, mut f: impl FnMut(&Bounds) -> Bounds) -> Self {
        match self {
            DrawingBounds::Empty => self.clone(),
            DrawingBounds::Unbounded => self.clone(),
            DrawingBounds::Bounded(b) => DrawingBounds::Bounded(f(b)),
        }
    }

    pub fn outset(&self, outset: &Outset) -> DrawingBounds {
        match self {
            DrawingBounds::Empty => DrawingBounds::Empty,
            DrawingBounds::Unbounded => DrawingBounds::Unbounded,
            DrawingBounds::Bounded(b) => match b.outset(outset) {
                Some(b) => DrawingBounds::Bounded(b),
                None => DrawingBounds::Empty,
            },
        }
    }

    pub fn intersect(a: &DrawingBounds, b: &DrawingBounds) -> DrawingBounds {
        match (a, b) {
            (DrawingBounds::Empty, _) => DrawingBounds::Empty,
            (_, DrawingBounds::Empty) => DrawingBounds::Empty,
            (DrawingBounds::Unbounded, b) => b.clone(),
            (a, DrawingBounds::Unbounded) => a.clone(),
            (DrawingBounds::Bounded(a), DrawingBounds::Bounded(b)) => {
                match Bounds::intersect(&a, &b) {
                    Some(intersection) => DrawingBounds::Bounded(intersection),
                    None => DrawingBounds::Empty,
                }
            }
        }
    }

    pub fn union_all(it: impl IntoIterator<Item = DrawingBounds>) -> DrawingBounds {
        it.into_iter()
            .fold(DrawingBounds::Empty, DrawingBounds::union)
    }
}

impl Union for DrawingBounds {
    fn union(a: Self, b: Self) -> Self {
        match (a, b) {
            (DrawingBounds::Empty, b) => b.clone(),
            (a, DrawingBounds::Empty) => a.clone(),
            (DrawingBounds::Unbounded, _) => DrawingBounds::Unbounded,
            (_, DrawingBounds::Unbounded) => DrawingBounds::Unbounded,
            (DrawingBounds::Bounded(a), DrawingBounds::Bounded(b)) => {
                DrawingBounds::Bounded(Bounds::union(a, b))
            }
        }
    }
}
