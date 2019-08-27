use crate::{Oval, Point, Radius, Rect, Vector};
use serde_tuple::*;

/// A circle at a center point with a radius.
#[derive(Clone, Serialize_tuple, Deserialize_tuple, PartialEq, Debug)]
pub struct Circle {
    pub center: Point,
    pub radius: Radius,
}

pub fn circle(center: impl Into<Point>, r: impl Into<Radius>) -> Circle {
    Circle::new(center.into(), r.into())
}

impl Circle {
    pub const fn new(center: Point, radius: Radius) -> Circle {
        Circle { center, radius }
    }

    pub fn to_oval(&self) -> Oval {
        let rv = Vector::new(*self.radius, *self.radius);
        Oval::new(&Rect::new(self.center - rv, self.center + rv))
    }
}
