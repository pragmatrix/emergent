use crate::functions::vector;
use crate::{Oval, Point, Radius, Rect};
use serde_tuple::*;

/// A circle at a center point with a radius.
#[derive(Clone, Serialize_tuple, Deserialize_tuple, PartialEq, Debug)]
pub struct Circle {
    pub center: Point,
    pub radius: Radius,
}

impl Circle {
    pub const fn new(center: Point, radius: Radius) -> Circle {
        Circle { center, radius }
    }

    pub fn to_oval(&self) -> Oval {
        let rv = vector(*self.radius, *self.radius);
        Oval(Rect::new(self.center - rv, self.center + rv))
    }
}
