use crate::{scalar, Extent, Vector};
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct Point(pub scalar, pub scalar);

impl Point {
    pub const fn new(left: scalar, top: scalar) -> Self {
        Self(left, top)
    }

    pub fn left(&self) -> scalar {
        self.0
    }

    pub fn top(&self) -> scalar {
        self.1
    }

    pub const ZERO: Point = Point::new(0.0, 0.0);

    pub fn to_vector(&self) -> Vector {
        Vector::new(self.left(), self.top())
    }
}

//
// Point <-> Point
//

impl Sub<Point> for Point {
    type Output = Vector;
    fn sub(self, rhs: Point) -> Vector {
        Vector::from((rhs.left() - self.left(), rhs.top() - self.top()))
    }
}

//
// Point <-> Extent
//

impl AddAssign<Extent> for Point {
    fn add_assign(&mut self, rhs: Extent) {
        self.0 += rhs.width();
        self.1 += rhs.height();
    }
}

impl Add<Extent> for Point {
    type Output = Self;
    fn add(mut self, rhs: Extent) -> Self {
        self += rhs;
        self
    }
}

impl SubAssign<Extent> for Point {
    fn sub_assign(&mut self, rhs: Extent) {
        self.0 -= rhs.width();
        self.1 -= rhs.height();
    }
}

impl Sub<Extent> for Point {
    type Output = Self;
    fn sub(mut self, rhs: Extent) -> Self {
        self -= rhs;
        self
    }
}

//
// Point <-> Vector
//

impl AddAssign<Vector> for Point {
    fn add_assign(&mut self, rhs: Vector) {
        self.0 += rhs.x();
        self.1 += rhs.y();
    }
}

impl Add<Vector> for Point {
    type Output = Self;
    fn add(mut self, rhs: Vector) -> Self {
        self += rhs;
        self
    }
}

impl SubAssign<Vector> for Point {
    fn sub_assign(&mut self, rhs: Vector) {
        self.0 -= rhs.x();
        self.1 -= rhs.y();
    }
}

impl Sub<Vector> for Point {
    type Output = Self;
    fn sub(mut self, rhs: Vector) -> Self {
        self -= rhs;
        self
    }
}

impl MulAssign<Vector> for Point {
    fn mul_assign(&mut self, rhs: Vector) {
        self.0 *= rhs.x();
        self.1 *= rhs.y();
    }
}

impl Mul<Vector> for Point {
    type Output = Self;
    fn mul(mut self, rhs: Vector) -> Self::Output {
        self *= rhs;
        self
    }
}

impl DivAssign<Vector> for Point {
    fn div_assign(&mut self, rhs: Vector) {
        self.0 /= rhs.x();
        self.1 /= rhs.y();
    }
}

impl Div<Vector> for Point {
    type Output = Self;
    fn div(mut self, rhs: Vector) -> Self::Output {
        self /= rhs;
        self
    }
}

//
// From
//

impl From<Vector> for Point {
    fn from(v: Vector) -> Self {
        Point::new(v.x(), v.y())
    }
}

impl From<(scalar, scalar)> for Point {
    fn from((x, y): (scalar, scalar)) -> Self {
        Self(x, y)
    }
}

impl From<(isize, isize)> for Point {
    fn from((x, y): (isize, isize)) -> Self {
        Self::from((x as scalar, y as scalar))
    }
}
