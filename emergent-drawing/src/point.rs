use crate::{scalar, Extent, Vector};
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct Point(pub scalar, pub scalar);

impl Point {
    pub fn left(&self) -> scalar {
        self.0
    }

    pub fn top(&self) -> scalar {
        self.1
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

//
// From
//

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
