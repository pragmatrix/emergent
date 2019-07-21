use crate::{scalar, Extent, Point, Vector};
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// Normalized bounds with a positive extent.
#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct Bounds(pub Point, pub Extent);

impl Bounds {
    pub const fn new(point: Point, extent: Extent) -> Self {
        Self(point, extent)
    }

    pub fn left(&self) -> scalar {
        self.0.left()
    }

    pub fn top(&self) -> scalar {
        self.0.top()
    }

    pub fn right(&self) -> scalar {
        self.0.left() + self.1.width()
    }

    pub fn bottom(&self) -> scalar {
        self.0.top() + self.1.height()
    }

    pub fn left_top(&self) -> Point {
        self.0
    }

    pub fn right_top(&self) -> Point {
        self.left_top() + Extent::new(self.extent().width(), 0.0)
    }

    pub fn right_bottom(&self) -> Point {
        self.left_top() + self.extent()
    }

    pub fn left_bottom(&self) -> Point {
        self.left_top() + Extent::new(0.0, self.extent().height())
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
        [
            self.left_top(),
            self.right_top(),
            self.right_bottom(),
            self.left_bottom(),
        ]
    }

    pub fn union(a: &Bounds, b: &Bounds) -> Bounds {
        let left = a.left().min(b.left());
        let top = a.top().min(b.top());
        let right = a.right().max(b.right());
        let bottom = a.bottom().max(b.bottom());
        Self::new(
            Point::new(left, top),
            Extent::new(right - left, bottom - top),
        )
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
