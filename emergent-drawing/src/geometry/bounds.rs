use crate::functions::point;
use crate::{scalar, Extent, Outset, Point, Vector};
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// A rectangle with a positive extent.
///
/// May be set to zero or one-dimensional bounds when width / height are 0.0.
/// Lower-dimensions are important for bounds computation of shapes like points and horizontal /
/// vertical lines that are zero or one-dimensional and get their two-dimensionality
/// from a Paint's stroke width, for example.
#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct Bounds(pub Point, pub Extent);

impl Bounds {
    pub const fn new(point: Point, extent: Extent) -> Self {
        Self(point, extent)
    }

    pub fn left(&self) -> scalar {
        self.0.x
    }

    pub fn top(&self) -> scalar {
        self.0.y
    }

    pub fn right(&self) -> scalar {
        self.0.x + self.1.width
    }

    pub fn bottom(&self) -> scalar {
        self.0.y + self.1.height
    }

    pub fn left_top(&self) -> Point {
        self.0
    }

    pub fn right_top(&self) -> Point {
        self.left_top() + Extent::new(self.extent().width, 0.0)
    }

    pub fn right_bottom(&self) -> Point {
        self.left_top() + self.extent()
    }

    pub fn left_bottom(&self) -> Point {
        self.left_top() + Extent::new(0.0, self.extent().height)
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
        let mut left = p1.x;
        let mut top = p1.y;
        let mut right = left;
        let mut bottom = top;
        points[1..].iter().for_each(|p| {
            let (x, y) = (p.x, p.y);
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

    /// Adds the outset to the bounds and returns a new bounds.
    ///
    /// Returns None if the outset is an inset and would reduce width or height
    /// below 0.0.
    #[must_use]
    pub fn outset(&self, outset: &Outset) -> Option<Bounds> {
        let l = self.left() - outset.left();
        let t = self.top() - outset.top();
        let r = self.right() + outset.right();
        let b = self.bottom() + outset.bottom();
        let width = r - l;
        let height = b - t;
        if width >= 0.0 && height >= 0.0 {
            Some(Bounds::new(point(l, t), Extent::new(width, height)))
        } else {
            None
        }
    }

    /// Returns the union of two bounds.
    pub fn union(a: &Bounds, b: &Bounds) -> Bounds {
        let left = a.left().min(b.left());
        let top = a.top().min(b.top());
        let right = a.right().max(b.right());
        let bottom = a.bottom().max(b.bottom());
        Self::new(point(left, top), Extent::new(right - left, bottom - top))
    }

    /// Returns the intersection of two bounds.
    ///
    /// None if they don't intersect.
    pub fn intersect(a: &Bounds, b: &Bounds) -> Option<Bounds> {
        let left = a.left().max(b.left());
        let top = a.top().max(b.top());
        let right = a.right().min(b.right());
        let bottom = a.bottom().min(b.bottom());
        if right > left && bottom > top {
            Some(Self::new(
                point(left, top),
                Extent::new(right - left, bottom - top),
            ))
        } else {
            None
        }
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
