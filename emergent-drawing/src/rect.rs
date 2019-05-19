use crate::{scalar, Point, Rect, Size};

impl Rect {
    pub fn is_empty(&self) -> bool {
        self.size().is_empty()
    }

    pub fn left(&self) -> scalar {
        self.left_top().left()
    }

    pub fn top(&self) -> scalar {
        self.left_top().top()
    }

    pub fn width(&self) -> scalar {
        self.size().width()
    }

    pub fn height(&self) -> scalar {
        self.size().height()
    }

    pub fn right(&self) -> scalar {
        self.left() + self.width()
    }

    pub fn bottom(&self) -> scalar {
        self.top() + self.height()
    }

    pub fn size(&self) -> Size {
        self.1
    }

    pub fn left_top(&self) -> Point {
        self.0
    }

    pub fn right_top(&self) -> Point {
        self.left_top() + Size::from((self.width(), 0.0))
    }

    pub fn right_bottom(&self) -> Point {
        self.left_top() + self.size()
    }

    pub fn left_bottom(&self) -> Point {
        self.left_top() + Size::from((0.0, self.height()))
    }

    pub fn center(&self) -> Point {
        self.left_top() + self.size() * 0.5
    }

    // Returns a rectangle that encloses two other rectangles.
    // This function _does_ include empty rectangles into the
    // resulting rectangle.
    pub fn union(a: &Rect, b: &Rect) -> Rect {
        Self::from_points_as_bounds(&[
            a.left_top(),
            a.right_bottom(),
            b.left_top(),
            b.right_bottom(),
        ])
        .unwrap()
    }

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

    // 0 points -> No Rect representation.
    // 1 point -> A zero sized rectangle at the point's position.
    pub fn from_points_as_bounds(points: &[Point]) -> Option<Self> {
        if points.is_empty() {
            return None;
        }

        let p1 = points[0];
        let mut left = p1.left();
        let mut top = p1.top();
        let mut right = left;
        let mut bottom = top;
        points[1..].iter().for_each(|p| {
            let (x, y) = (p.x(), p.y());
            left = left.min(x);
            top = top.min(y);
            right = right.max(x);
            bottom = bottom.max(y);
        });
        Some(Rect::from((
            Point::from((left, top)),
            Size::from((right - left, bottom - top)),
        )))
    }
}

impl From<(Point, Size)> for Rect {
    fn from((p, s): (Point, Size)) -> Self {
        Rect(p, s)
    }
}
