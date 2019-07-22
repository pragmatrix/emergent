use crate::{scalar, Extent, Vector};
use serde_tuple::*;
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Copy, Clone, Serialize_tuple, Deserialize_tuple, PartialEq, Default, Debug)]
pub struct Point {
    pub x: scalar,
    pub y: scalar,
}

impl Point {
    pub const ZERO: Point = Point::new(0.0, 0.0);

    pub const fn new(x: scalar, y: scalar) -> Self {
        Point { x, y }
    }

    pub fn to_vector(&self) -> Vector {
        Vector::new(self.x, self.y)
    }
}

//
// Point <-> Point
//

impl Sub<Point> for Point {
    type Output = Vector;
    fn sub(self, rhs: Point) -> Vector {
        Vector::from((rhs.x - self.x, rhs.y - self.x))
    }
}

//
// Point <-> Extent
//

impl AddAssign<Extent> for Point {
    fn add_assign(&mut self, rhs: Extent) {
        self.x += rhs.width();
        self.y += rhs.height();
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
        self.x -= rhs.width();
        self.y -= rhs.height();
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
        self.x += rhs.x;
        self.y += rhs.y;
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
        self.x -= rhs.x;
        self.y -= rhs.y;
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
        self.x *= rhs.x;
        self.y *= rhs.y;
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
        self.x /= rhs.x;
        self.y /= rhs.y;
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
        Point::new(v.x, v.y)
    }
}

impl From<(scalar, scalar)> for Point {
    fn from((x, y): (scalar, scalar)) -> Self {
        Point::new(x, y)
    }
}

impl From<(isize, isize)> for Point {
    fn from((x, y): (isize, isize)) -> Self {
        Self::from((x as scalar, y as scalar))
    }
}
