use crate::{
    Arc, Bounds, Circle, Clip, Draw, Drawing, Extent, Font, Line, Outset, Oval, Point, Polygon,
    Rect, RoundedRect, Shape, Vector,
};

pub trait MeasureText {
    // Measure the given text resulting a bounds relative to it's baseline starting point
    // positioned at 0,0.
    fn measure_text(&self, str: &str, font: &Font) -> Bounds;
}

#[derive(Clone, PartialEq, Debug)]
pub enum DrawingBounds {
    /// Nothing to be drawn.
    Empty,
    /// May fill everything given to it.
    Unbounded,
    /// Bounded.
    Bounded(Bounds),
}

pub trait FastBounds {
    fn fast_bounds(&self) -> Bounds;
}

pub trait ComplexFastBounds {
    fn fast_bounds(&self, text: &impl MeasureText) -> Bounds;
}

pub trait DrawingFastBounds {
    fn fast_bounds(&self, text: &impl MeasureText) -> DrawingBounds;
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
        Bounds::from_points(&[self.0, self.1]).unwrap()
    }
}

impl FastBounds for Rect {
    fn fast_bounds(&self) -> Bounds {
        Bounds::from_points(&[self.0, self.0 + self.1]).unwrap()
    }
}

impl FastBounds for Polygon {
    fn fast_bounds(&self) -> Bounds {
        Bounds::from_points(&self.0).unwrap()
    }
}

impl FastBounds for Oval {
    fn fast_bounds(&self) -> Bounds {
        self.0.fast_bounds()
    }
}

impl FastBounds for Circle {
    fn fast_bounds(&self) -> Bounds {
        let r = self.1;
        let r = Vector::from((*r, *r));
        let p = self.0 - r;
        Bounds::from((p, (r * 2.0).into()))
    }
}

impl FastBounds for RoundedRect {
    fn fast_bounds(&self) -> Bounds {
        self.0.fast_bounds()
    }
}

impl FastBounds for Arc {
    fn fast_bounds(&self) -> Bounds {
        self.0.fast_bounds()
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
    fn fast_bounds(&self, measure_text: &impl MeasureText) -> Bounds {
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
            // TODO: empty text?
            Shape::Text(t) => measure_text.measure_text(&t.1, &t.2),
        }
    }
}

impl DrawingFastBounds for Draw {
    fn fast_bounds(&self, text: &impl MeasureText) -> DrawingBounds {
        match self {
            Draw::Paint(_, _) => DrawingBounds::Unbounded,
            Draw::Shapes(shapes, paint) => DrawingBounds::union_all(
                shapes
                    .iter()
                    .map(|s| DrawingBounds::Bounded(s.fast_bounds(text))),
            )
            .outset(&paint.fast_outset()),
            Draw::Clipped(clip, drawing) => {
                let clip = DrawingBounds::Bounded(clip.fast_bounds());
                let drawing = drawing.fast_bounds(text);
                DrawingBounds::intersect(&clip, &drawing)
            }
            Draw::Transformed(transform, drawing) => {
                let drawing_bounds = drawing.fast_bounds(text);
                drawing_bounds.map_bounded(|b| transform.to_matrix().map_bounds(*b))
            }
        }
    }
}

impl DrawingFastBounds for Drawing {
    fn fast_bounds(&self, text: &impl MeasureText) -> DrawingBounds {
        DrawingBounds::union_all(self.0.iter().map(|d| d.fast_bounds(text)))
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

    pub fn union(a: &DrawingBounds, b: &DrawingBounds) -> DrawingBounds {
        match (a, b) {
            (DrawingBounds::Empty, b) => b.clone(),
            (a, DrawingBounds::Empty) => a.clone(),
            (DrawingBounds::Unbounded, _) => DrawingBounds::Unbounded,
            (_, DrawingBounds::Unbounded) => DrawingBounds::Unbounded,
            (DrawingBounds::Bounded(a), DrawingBounds::Bounded(b)) => {
                DrawingBounds::Bounded(Bounds::union(&a, &b))
            }
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

    pub fn union_all<'a>(it: impl IntoIterator<Item = DrawingBounds>) -> DrawingBounds {
        it.into_iter()
            .fold(DrawingBounds::Empty, |a, b| DrawingBounds::union(&a, &b))
    }
}
