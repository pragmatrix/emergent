use emergent_drawing::{scalar, Vector};

pub trait Interpolated {
    fn interpolated(&self, to: &Self, t: scalar) -> Self;
}

impl Interpolated for scalar {
    fn interpolated(&self, to: &Self, t: f64) -> Self {
        *self + (*to - *self) * t
    }
}

impl Interpolated for Vector {
    fn interpolated(&self, to: &Self, t: scalar) -> Self {
        Vector::new(self.x.interpolated(&to.x, t), self.y.interpolated(&to.y, t))
    }
}
