use crate::{
    Arc, Bounds, Circle, Font, Line, Oval, Point, Polygon, Rect, RoundedRect, Shape, Vector,
};

pub trait MeasureText {
    // Measure the given text resulting a bounds relative to it's baseline starting point
    // positioned at 0,0.
    fn measure_text(&self, str: &str, font: &Font) -> Bounds;
}

pub trait FastBounds {
    fn fast_bounds(&self) -> Bounds;
}

pub trait ComplexFastBounds {
    fn fast_bounds(&self, text: impl MeasureText) -> Bounds;
}

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
        let r = Vector::from(((self.1).0, (self.1).0));
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

impl ComplexFastBounds for Shape {
    fn fast_bounds(&self, measure_text: impl MeasureText) -> Bounds {
        match self {
            Shape::Point(p) => p.fast_bounds(),
            Shape::Line(l) => l.fast_bounds(),
            Shape::Polygon(p) => p.fast_bounds(),
            Shape::Rect(r) => r.fast_bounds(),
            Shape::Oval(o) => o.fast_bounds(),
            Shape::RoundedRect(rr) => rr.fast_bounds(),
            Shape::Circle(c) => c.fast_bounds(),
            Shape::Arc(a) => a.fast_bounds(),
            Shape::Path(p) => p.fast_bounds(),
            Shape::Image(_, _, target) => target.fast_bounds(),
            Shape::Text(t) => measure_text.measure_text(&t.1, &t.2),
        }
    }
}
