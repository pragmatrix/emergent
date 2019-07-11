use crate::{scalar, Bounds, Outset, Point, Vector};
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// A rectangle, defined by a point and a vector.
#[derive(Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct Rect(pub Point, pub Vector);

impl Rect {
    pub fn from_points(lt: Point, rb: Point) -> Rect {
        Rect::from((lt, rb - lt))
    }

    pub fn is_empty(&self) -> bool {
        self.size().is_zero()
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
        self.left_top() + self.size()
    }

    /// The point at the left / bottom corner of the rectangle.
    pub fn left_bottom(&self) -> Point {
        (self.left(), self.bottom()).into()
    }

    /// The left edge of the rectangle (or the right if width is negative).
    pub fn left(&self) -> scalar {
        self.0.left()
    }

    /// The top edge of the rectangle (or the bottom if height is negative).
    pub fn top(&self) -> scalar {
        self.0.top()
    }

    /// The right edge of the rectangle (or the left if width is negative).
    pub fn right(&self) -> scalar {
        self.left() + self.width()
    }

    /// The bottom edge of the rectangle (or the top if height is negative).
    pub fn bottom(&self) -> scalar {
        self.top() + self.height()
    }

    /// The width, may be negative.
    pub fn width(&self) -> scalar {
        self.size().x()
    }

    /// The height, may be negative.
    pub fn height(&self) -> scalar {
        self.size().y()
    }

    pub fn size(&self) -> Vector {
        self.1
    }

    pub fn center(&self) -> Point {
        self.0 + (self.size() * 0.5)
    }

    // Returns a rectangle that encloses two other rectangles.
    // This function _does_ treat the location of empty rectangles
    // as a point inside the bounds.
    pub fn union(a: &Rect, b: &Rect) -> Rect {
        Bounds::from_points(&[
            a.left_top(),
            a.right_bottom(),
            b.left_top(),
            b.right_bottom(),
        ])
        .unwrap()
        .into()
    }

    // If the size vector is positive, returns 4 points in the following order:
    // 0--1
    // |  |
    // 3--2
    pub fn points(&self) -> [Point; 4] {
        [
            self.left_top(),
            self.right_top(),
            self.right_bottom(),
            self.left_bottom(),
        ]
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
        let h_dir = sz.x().signum();
        let v_dir = sz.y().signum();
        let l = rhs.left() * h_dir;
        let t = rhs.top() * v_dir;
        let r = rhs.right() * h_dir;
        let b = rhs.bottom() * v_dir;
        self.0 -= Vector::from((l, t));
        self.1 += Vector::from((r, b));
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
        Rect(p, size)
    }
}
