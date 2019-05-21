use crate::{scalar, Point, Size};
use std::ops;

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
// From tuple conversions
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
