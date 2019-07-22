use crate::{scalar, Bounds, Outset, Point, Vector};
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// A rectangle, defined by two points.
#[derive(Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct Rect(pub Point, pub Point);

impl Rect {
    pub fn new(p1: impl Into<Point>, p2: impl Into<Point>) -> Rect {
        Rect(p1.into(), p2.into())
    }

    pub fn is_empty(&self) -> bool {
        self.0 == self.1
    }

    /// The point at the left / top corner of the rectangle,
    /// if the width / height is positive.
    pub fn left_top(&self) -> Point {
        self.0
    }

    /// The point at the right / top corner of the rectangle,
    pub fn right_top(&self) -> Point {
        (self.right(), self.top()).into()
    }

    /// The point at the right / bottom corner of the rectangle,
    /// if the width / height is positive.
    pub fn right_bottom(&self) -> Point {
        self.1
    }

    /// The point at the left / bottom corner of the rectangle.
    pub fn left_bottom(&self) -> Point {
        (self.left(), self.bottom()).into()
    }

    /// The left edge of the rectangle (or the right if width is negative).
    pub fn left(&self) -> scalar {
        self.0.x
    }

    /// The top edge of the rectangle (or the bottom if height is negative).
    pub fn top(&self) -> scalar {
        self.0.y
    }

    /// The right edge of the rectangle (or the left if width is negative).
    pub fn right(&self) -> scalar {
        self.1.x
    }

    /// The bottom edge of the rectangle (or the top if height is negative).
    pub fn bottom(&self) -> scalar {
        self.1.y
    }

    /// The width, may be negative.
    pub fn width(&self) -> scalar {
        self.1.x - self.0.x
    }

    /// The height, may be negative.
    pub fn height(&self) -> scalar {
        self.1.y - self.0.y
    }

    pub fn size(&self) -> Vector {
        Vector::new(self.width(), self.height())
    }

    pub fn center(&self) -> Point {
        (self.0 + self.1.to_vector()) * Vector::new(0.5, 0.5)
    }

    /// Returns a rectangle that encloses two other rectangles.
    pub fn union(a: &Rect, b: &Rect) -> Rect {
        let l = a.left().min(b.left());
        let t = a.top().min(b.top());
        let r = a.right().max(b.right());
        let b = a.bottom().max(b.bottom());
        Rect::new(Point::new(l, t), Point::new(r, b))
    }

    /// If they intersect, returns the intersection of two rectangles.
    pub fn intersect(a: &Rect, b: &Rect) -> Option<Rect> {
        let l = a.left().max(b.left());
        let t = a.top().max(b.top());
        let r = a.right().min(b.right());
        let b = a.bottom().min(b.bottom());
        if r > l && b > t {
            Some(Rect::new(Point::new(l, t), Point::new(r, b)))
        } else {
            None
        }
    }

    // If the size vector is positive, returns a quadriteral in the following order:
    // 0--1
    // |  |
    // 3--2
    pub fn to_quad(&self) -> [Point; 4] {
        [
            self.left_top(),
            self.right_top(),
            self.right_bottom(),
            self.left_bottom(),
        ]
    }

    pub fn bounds(&self) -> Bounds {
        Bounds::from_points(&[self.0, self.1]).unwrap()
    }
}

impl AddAssign<Outset> for Rect {
    fn add_assign(&mut self, rhs: Outset) {
        // Outset is applied edge-relative to the rect, meaning
        // that when the size in one dimension is negative, the same edge is
        // used but extended in the other direction.
        // For example: The left edge stays the left edge, even if it's
        // on the other side because of a negative horizontal size of the rect, so
        // positive out-sets _always_ make the area of the rect larger.
        // This makes sure that applying an outset to a rect is the same as
        // flipping the rect, adding the outset, and flipping it back.
        let sz = self.size();
        let h_dir = sz.x.signum();
        let v_dir = sz.y.signum();
        let l = rhs.left() * h_dir;
        let t = rhs.top() * v_dir;
        let r = rhs.right() * h_dir;
        let b = rhs.bottom() * v_dir;
        let lt = Vector::from((l, t));
        self.0 -= lt;
        self.1 += lt + Vector::from((r, b));
    }
}

impl Add<Outset> for Rect {
    type Output = Rect;
    fn add(mut self, rhs: Outset) -> Self::Output {
        self += rhs;
        self
    }
}

impl SubAssign<Outset> for Rect {
    fn sub_assign(&mut self, rhs: Outset) {
        *self += -rhs;
    }
}

impl Sub<Outset> for Rect {
    type Output = Self;
    fn sub(mut self, rhs: Outset) -> Self::Output {
        self -= rhs;
        self
    }
}

impl From<Bounds> for Rect {
    fn from(b: Bounds) -> Self {
        Rect::from((b.left_top(), b.extent().into()))
    }
}

impl From<(Point, Vector)> for Rect {
    fn from((p, size): (Point, Vector)) -> Self {
        Rect::new(p, size)
    }
}
