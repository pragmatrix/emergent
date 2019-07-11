use crate::{Extent, Point};

/// Normalized bounds with a positive extent.
pub struct Bounds(pub Point, pub Extent);

impl Bounds {
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
}

impl From<(Point, Extent)> for Bounds {
    fn from((p, e): (Point, Extent)) -> Self {
        Bounds(p, e)
    }
}
