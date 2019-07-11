use crate::{scalar, Extent};
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct Vector(pub scalar, pub scalar);

impl Vector {
    pub fn new(x: scalar, y: scalar) -> Self {
        Self::from((x, y))
    }

    pub fn x(&self) -> scalar {
        self.0
    }

    pub fn y(&self) -> scalar {
        self.1
    }

    /// Returns true if both scalars are >= 0.
    pub fn is_positive(&self) -> bool {
        self.0 >= 0.0 && self.1 >= 0.0
    }

    /// Returns true if the vector's length is zero.
    /// TODO: rename to is_origin()?
    pub fn is_zero(&self) -> bool {
        self.0 == 0.0 && self.1 == 0.0
    }

    pub fn length(&self) -> scalar {
        (self.0 * self.0 + self.1 * self.1).sqrt()
    }
}

//
// From
//

impl From<(scalar, scalar)> for Vector {
    fn from((x, y): (scalar, scalar)) -> Self {
        Self(x, y)
    }
}

impl From<(isize, isize)> for Vector {
    fn from((w, h): (isize, isize)) -> Self {
        Self::from((w as scalar, h as scalar))
    }
}

impl From<Extent> for Vector {
    fn from(extent: Extent) -> Self {
        Vector::from((extent.width(), extent.height()))
    }
}

//
// Vector <-> Vector
//

impl Add for Vector {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self::Output {
        self += rhs;
        self
    }
}

impl AddAssign for Vector {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
        self.1 += rhs.1;
    }
}

impl Sub for Vector {
    type Output = Self;
    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

impl SubAssign for Vector {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
        self.1 -= rhs.1;
    }
}

impl Neg for Vector {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.x(), -self.y())
    }
}

//
// Vector <-> scalar
//

impl Mul<scalar> for Vector {
    type Output = Vector;
    fn mul(mut self, rhs: scalar) -> Self::Output {
        self *= rhs;
        self
    }
}

impl MulAssign<scalar> for Vector {
    fn mul_assign(&mut self, rhs: scalar) {
        self.0 *= rhs;
        self.1 *= rhs;
    }
}

impl Div<scalar> for Vector {
    type Output = Vector;
    fn div(mut self, rhs: f32) -> Self::Output {
        self /= rhs;
        self
    }
}

impl DivAssign<scalar> for Vector {
    fn div_assign(&mut self, rhs: scalar) {
        self.0 /= rhs;
        self.1 /= rhs;
    }
}
