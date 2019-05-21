use crate::{scalar, Padding, Point, Size, Vector};
use std::ops;

impl Vector {
    pub fn x(&self) -> scalar {
        self.0
    }

    pub fn y(&self) -> scalar {
        self.1
    }
}

impl Point {
    pub fn left(&self) -> scalar {
        self.1
    }

    pub fn top(&self) -> scalar {
        self.0
    }

    pub fn x(&self) -> scalar {
        self.left()
    }

    pub fn y(&self) -> scalar {
        self.top()
    }
}

impl Size {
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

impl Padding {
    pub fn left(&self) -> scalar {
        self.0[0]
    }

    pub fn top(&self) -> scalar {
        self.0[1]
    }

    pub fn right(&self) -> scalar {
        self.0[2]
    }

    pub fn bottom(&self) -> scalar {
        self.0[3]
    }

    /// Vector to subtract from the left / top of a rectangle to adjust the padding.
    pub fn left_top(&self) -> Vector {
        Vector::from((-self.left(), -self.top()))
    }

    /// Vector to add to the right bottom of a rectangle to adjust the padding.
    pub fn right_bottom(&self) -> Vector {
        Vector::from((self.right(), self.bottom()))
    }
}

//
// Size <-> scalar
//

impl ops::Mul<scalar> for Size {
    type Output = Self;

    fn mul(self, scalar: scalar) -> Self {
        Size::from((self.width() * scalar, self.height() * scalar))
    }
}

impl ops::Div<scalar> for Size {
    type Output = Self;

    fn div(self, scalar: scalar) -> Self {
        Size::from((self.width() / scalar, self.height() / scalar))
    }
}

//
// Point <-> Point
//

impl ops::Sub<Point> for Point {
    type Output = Vector;

    fn sub(self, rhs: Point) -> Vector {
        Vector::from((rhs.left() - self.left(), rhs.top() - self.top()))
    }
}

//
// Point <-> Size
//

impl ops::Add<Size> for Point {
    type Output = Self;

    fn add(self, rhs: Size) -> Self {
        Point(self.left() + rhs.width(), self.y() + rhs.height())
    }
}

impl ops::Sub<Size> for Point {
    type Output = Self;

    fn sub(self, rhs: Size) -> Self {
        Point(self.left() - rhs.width(), self.y() - rhs.height())
    }
}

//
// Point <-> Vector
//

impl ops::Add<Vector> for Point {
    type Output = Self;

    fn add(self, rhs: Vector) -> Self {
        Self(self.left() + rhs.x(), self.y() + rhs.y())
    }
}

impl ops::Sub<Vector> for Point {
    type Output = Self;

    fn sub(self, rhs: Vector) -> Self {
        Self(self.left() - rhs.x(), self.y() - rhs.y())
    }
}

//
// Vector
//

impl From<(scalar, scalar)> for Vector {
    fn from((x, y): (scalar, scalar)) -> Self {
        Self(x, y)
    }
}

impl From<Size> for Vector {
    fn from(sz: Size) -> Self {
        Vector::from((sz.width(), sz.height()))
    }
}

//
// Point
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

//
// Size
//

impl From<(scalar, scalar)> for Size {
    fn from((x, y): (scalar, scalar)) -> Self {
        Self(x, y)
    }
}

impl From<(isize, isize)> for Size {
    fn from((w, h): (isize, isize)) -> Self {
        Self::from((w as scalar, h as scalar))
    }
}

impl From<Vector> for Size {
    fn from(v: Vector) -> Self {
        Size::from((v.x(), v.y()))
    }
}

//
// Padding
//

impl From<scalar> for Padding {
    fn from(padding: f32) -> Self {
        Self::from((padding, padding))
    }
}

impl From<(scalar, scalar)> for Padding {
    fn from((padding_h, padding_v): (scalar, scalar)) -> Self {
        Self([padding_h, padding_v, padding_h, padding_v])
    }
}
