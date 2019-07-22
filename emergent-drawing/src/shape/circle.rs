use crate::{Oval, Point, Radius, Rect, Vector};
use serde_tuple::*;

/// A circle at a center point with a radius.
#[derive(Clone, Serialize_tuple, Deserialize_tuple, PartialEq, Debug)]
pub struct Circle {
    pub center: Point,
    pub radius: Radius,
}

impl Circle {
    pub fn to_oval(&self) -> Oval {
        let rv = Vector::new(*self.radius, *self.radius);
        Oval(Rect::new(self.center - rv, rv * 2.0))
    }
}
