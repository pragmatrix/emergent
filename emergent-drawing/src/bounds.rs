use crate::{Extent, Point, Rect, Vector};
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// Normalized bounds with a positive extent.
#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct Bounds(pub Point, pub Extent);

impl Bounds {
    pub const fn new(point: Point, extent: Extent) -> Self {
        Self(point, extent)
    }

    pub fn left_top(&self) -> Point {
        self.0
    }

    pub fn extent(&self) -> Extent {
        self.1
    }

    // 0 points -> No Rect representation.
    // 1 point -> A zero sized rectangle at the point's position.
    pub fn from_points(points: &[Point]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let p1 = points[0];
        let mut left = p1.left();
        let mut top = p1.top();
        let mut right = left;
        let mut bottom = top;
        points[1..].iter().for_each(|p| {
            let (x, y) = (p.left(), p.top());
            left = left.min(x);
            top = top.min(y);
            right = right.max(x);
            bottom = bottom.max(y);
        });
        Some(Self(
            Point::from((left, top)),
            Extent::from((right - left, bottom - top)),
        ))
    }

    pub fn to_quad(&self) -> [Point; 4] {
        // TODO: I guess Rect is a higher-level data type than Bounds and should not be used here.
        Rect::from(*self).to_quad()
    }
}

impl From<(Point, Extent)> for Bounds {
    fn from((p, e): (Point, Extent)) -> Self {
        Bounds(p, e)
    }
}

impl AddAssign<Vector> for Bounds {
    fn add_assign(&mut self, rhs: Vector) {
        self.0 += rhs;
    }
}

impl Add<Vector> for Bounds {
    type Output = Self;
    fn add(mut self, rhs: Vector) -> Self::Output {
        self += rhs;
        self
    }
}

impl SubAssign<Vector> for Bounds {
    fn sub_assign(&mut self, rhs: Vector) {
        self.0 -= rhs
    }
}

impl Sub<Vector> for Bounds {
    type Output = Self;
    fn sub(mut self, rhs: Vector) -> Self::Output {
        self -= rhs;
        self
    }
}
