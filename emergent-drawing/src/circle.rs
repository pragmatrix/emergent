use crate::{Oval, Point, Radius, Rect, Vector};
use serde::{Deserialize, Serialize};

/// A circle at a center point with a radius.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Circle(pub Point, pub Radius);

impl Circle {
    pub fn to_oval(&self) -> Oval {
        let rv = Vector::new(*self.1, *self.1);
        Oval(Rect::new(self.0 - rv, rv * 2.0))
    }
}
