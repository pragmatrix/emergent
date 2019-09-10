use crate::{point::point, scalar, vector::vector, Bounds, Contains, Outset, Point, Vector};
use serde_tuple::*;
use std::ops::{Add, AddAssign, Sub, SubAssign};

/// A rectangle, defined by two points.
#[derive(Clone, Serialize_tuple, Deserialize_tuple, PartialEq, Default, Debug)]
pub struct Rect {
    pub left: scalar,
    pub top: scalar,
    pub right: scalar,
    pub bottom: scalar,
}

pub fn rect(p: impl Into<Point>, v: impl Into<Vector>) -> Rect {
    let p = p.into();
    Rect::new(p, p + v.into())
}

impl Rect {
    pub const fn new(p1: Point, p2: Point) -> Rect {
        Rect {
            left: p1.x,
            top: p1.y,
            right: p2.x,
            bottom: p2.y,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.left == self.right && self.top == self.bottom
    }

    /// The point at the left / top corner of the rectangle,
    /// if the width / height is positive.
    pub fn left_top(&self) -> Point {
        (self.left, self.top).into()
    }

    /// The point at the right / top corner of the rectangle,
    pub fn right_top(&self) -> Point {
        (self.right, self.top).into()
    }

    /// The point at the right / bottom corner of the rectangle,
    /// if the width / height is positive.
    pub fn right_bottom(&self) -> Point {
        (self.right, self.bottom).into()
    }

    /// The point at the left / bottom corner of the rectangle.
    pub fn left_bottom(&self) -> Point {
        (self.left, self.bottom).into()
    }

    /// The width, may be negative.
    pub fn width(&self) -> scalar {
        self.right - self.left
    }

    /// The height, may be negative.
    pub fn height(&self) -> scalar {
        self.bottom - self.top
    }

    pub fn size(&self) -> Vector {
        (self.width(), self.height()).into()
    }

    pub fn center(&self) -> Point {
        (self.left_top() + self.right_bottom().to_vector()) * vector(0.5, 0.5)
    }

    /// Returns a rectangle that encloses two other rectangles.
    pub fn union(a: &Rect, b: &Rect) -> Rect {
        let l = a.left.min(b.left);
        let t = a.top.min(b.top);
        let r = a.right.max(b.right);
        let b = a.bottom.max(b.bottom);
        Rect::new(point(l, t), point(r, b))
    }

    /// If they intersect, returns the intersection of two rectangles.
    pub fn intersect(a: &Rect, b: &Rect) -> Option<Rect> {
        let l = a.left.max(b.left);
        let t = a.top.max(b.top);
        let r = a.right.min(b.right);
        let b = a.bottom.min(b.bottom);
        if r > l && b > t {
            Some(Rect::new(point(l, t), point(r, b)))
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
        Bounds::from_points(&[self.left_top(), self.right_bottom()]).unwrap()
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
        self.left -= lt.x;
        self.top -= lt.y;
        let rb = lt + Vector::from((r, b));
        self.right += rb.x;
        self.bottom += rb.y;
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
        Rect::from((b.left_top(), b.extent.into()))
    }
}

impl From<(Point, Vector)> for Rect {
    fn from((p, size): (Point, Vector)) -> Self {
        rect(p, size)
    }
}

impl Contains<Point> for Rect {
    fn contains(&self, p: Point) -> bool {
        let (x, y) = (p.x, p.y);
        x >= self.left && x < self.right && y >= self.top && y < self.bottom
    }
}

impl Contains<&Rect> for Rect {
    fn contains(&self, r: &Rect) -> bool {
        self.left <= r.left && self.top <= r.top && self.right >= r.right && self.bottom >= r.bottom
    }
}
