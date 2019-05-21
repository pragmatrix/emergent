use crate::{Radius, Rect, RoundedRect, Vector};

impl From<(Rect, Radius)> for RoundedRect {
    fn from((rect, Radius(radius)): (Rect, Radius)) -> Self {
        let v = Vector(radius, radius);
        RoundedRect::from((rect, v))
    }
}

impl From<(Rect, Vector)> for RoundedRect {
    fn from((rect, v): (Rect, Vector)) -> Self {
        RoundedRect(rect, [v, v, v, v])
    }
}
