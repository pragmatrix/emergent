use crate::{scalar, Vector};
use serde::{Deserialize, Serialize};
use std::ops::{Mul, MulAssign};

/// An extent, 0 or positive width / height.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct Extent(scalar, scalar);

impl Extent {
    pub fn width(&self) -> scalar {
        self.0
    }

    pub fn height(&self) -> scalar {
        self.1
    }

    pub fn is_empty(&self) -> bool {
        self.width() == 0.0 || self.height() == 0.0
    }
}

//
// Extent <-> scalar
//

impl MulAssign<scalar> for Extent {
    fn mul_assign(&mut self, rhs: f32) {
        assert!(rhs >= 0.0);
        self.0 *= rhs;
        self.1 *= rhs;
    }
}

impl Mul<scalar> for Extent {
    type Output = Self;
    fn mul(mut self, rhs: f32) -> Self {
        self *= rhs;
        self
    }
}

//
// From
//

impl From<(scalar, scalar)> for Extent {
    fn from((x, y): (scalar, scalar)) -> Self {
        assert!(x >= 0.0 && y >= 0.0);
        Self(x, y)
    }
}

impl From<(isize, isize)> for Extent {
    fn from((w, h): (isize, isize)) -> Self {
        Self::from((w as scalar, h as scalar))
    }
}

impl From<Vector> for Extent {
    fn from(v: Vector) -> Self {
        Extent::from((v.x(), v.y()))
    }
}
