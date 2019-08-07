use crate::{scalar, Extent};
use serde_tuple::*;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

#[derive(Copy, Clone, Serialize_tuple, Deserialize_tuple, PartialEq, Default, Debug)]
pub struct Vector {
    pub x: scalar,
    pub y: scalar,
}

pub fn vector(x: scalar, y: scalar) -> Vector {
    Vector::new(x, y)
}

impl Vector {
    pub const ZERO: Vector = Vector::new(0.0, 0.0);

    pub const fn new(x: scalar, y: scalar) -> Self {
        Vector { x, y }
    }

    /// Returns true if both scalars are >= 0.
    pub fn is_positive(&self) -> bool {
        self.x >= 0.0 && self.y >= 0.0
    }

    /// Returns true if the vector's length is zero.
    /// TODO: rename to is_origin()?
    pub fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }

    pub fn length(&self) -> scalar {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    pub fn set_length(&mut self, length: scalar) -> bool {
        let (x, y) = (self.x, self.y);
        let dmag = (x * x + y * y).sqrt();
        let dscale = length / dmag;
        let x = x * dscale;
        let y = y * dscale;
        if !x.is_finite() || !y.is_finite() {
            *self = Vector::ZERO;
            return false;
        }
        *self = vector(x, y);
        return true;
    }

    pub fn dot_product(a: &Vector, b: &Vector) -> scalar {
        a.x * b.x + a.y * b.y
    }

    pub fn cross_product(a: &Vector, b: &Vector) -> scalar {
        a.x * b.y - a.y * b.x
    }
}

//
// From
//

impl From<(scalar, scalar)> for Vector {
    fn from((x, y): (scalar, scalar)) -> Self {
        Self::new(x, y)
    }
}

impl From<(isize, isize)> for Vector {
    fn from((w, h): (isize, isize)) -> Self {
        Self::from((w as scalar, h as scalar))
    }
}

impl From<Extent> for Vector {
    fn from(extent: Extent) -> Self {
        Vector::from((extent.width, extent.height))
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
        self.x += rhs.x;
        self.y += rhs.y;
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
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Neg for Vector {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self::new(-self.x, -self.y)
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
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Div<scalar> for Vector {
    type Output = Vector;
    fn div(mut self, rhs: scalar) -> Self::Output {
        self /= rhs;
        self
    }
}

impl DivAssign<scalar> for Vector {
    fn div_assign(&mut self, rhs: scalar) {
        self.x /= rhs;
        self.y /= rhs;
    }
}
