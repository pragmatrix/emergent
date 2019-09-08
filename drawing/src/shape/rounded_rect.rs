use crate::{scalar, Contains, Extent, Point, Radius, Rect, Scalar, Vector};
use serde_tuple::*;

/// A rounded rectangle.
// TODO: Optimize representation for simple cases?
// Corners are starting at the upper left and follow clockwise.
#[derive(Clone, Serialize_tuple, Deserialize_tuple, PartialEq, Debug)]
pub struct RoundedRect {
    rect: Rect,
    radii: [Extent; 4],
}

impl RoundedRect {
    pub fn new(r: &Rect, radii: &[Extent; 4]) -> Self {
        Self {
            rect: r.clone(),
            radii: *radii,
        }
    }

    /// The rect.
    pub fn rect(&self) -> &Rect {
        &self.rect
    }

    /// The corner radii.
    pub fn corner_radii(&self) -> &[Extent; 4] {
        &self.radii
    }

    pub fn is_rect(&self) -> bool {
        let zero: [Extent; 4] = Default::default();
        self.radii == zero
    }

    /// Bounds, same as `self.rect()`.
    pub fn bounds(&self) -> &Rect {
        self.rect()
    }

    const UPPER_LEFT: usize = 0;
    const UPPER_RIGHT: usize = 1;
    const LOWER_RIGHT: usize = 2;
    const LOWER_LEFT: usize = 3;

    /// Returns corner rectangles in the following order:
    ///
    /// 0-1
    /// | |
    /// 3-2
    pub fn to_corner_rects(&self) -> [Rect; 4] {
        let [r0, r1, r2, r3] = *self.corner_radii();
        [
            Rect::from((self.rect.left_top(), r0.into())),
            Rect::from((
                self.rect.right_top() - Vector::new(r1.width, 0.0),
                r1.into(),
            )),
            Rect::from((self.rect.right_bottom() - r2, r2.into())),
            Rect::from((
                self.rect.left_bottom() - Vector::new(0.0, r3.height),
                r3.into(),
            )),
        ]
    }

    /// Returns a polygon representation of the rounded rect.
    /// /0--1\
    /// 7    2
    /// |    |
    /// 6    3
    /// \5--4/
    ///
    pub fn to_points(&self) -> [Point; 8] {
        let cr = self.to_corner_rects();
        [
            cr[0].right_top(),
            cr[1].left_top(),
            cr[1].right_bottom(),
            cr[2].right_top(),
            cr[2].left_bottom(),
            cr[3].right_bottom(),
            cr[3].left_top(),
            cr[0].left_bottom(),
        ]
    }

    // Skia: 6d1c0d4196f19537cc64f74bacc7d123de3be454
    pub(crate) fn check_corner_containment(&self, x: scalar, y: scalar) -> bool {
        let canonical_pt; // (x,y) translated to one of the quadrants
        let index: usize;

        if x < self.rect.left + self.radii[Self::UPPER_LEFT].width
            && y < self.rect.top + self.radii[Self::UPPER_LEFT].height
        {
            // UL corner
            index = Self::UPPER_LEFT;
            canonical_pt = Point::new(
                x - (self.rect.left + self.radii[index].width),
                y - (self.rect.top + self.radii[index].height),
            );
            debug_assert!(canonical_pt.x < 0.0 && canonical_pt.y < 0.0);
        } else if x < self.rect.left + self.radii[Self::LOWER_LEFT].width
            && y > self.rect.bottom - self.radii[Self::LOWER_LEFT].height
        {
            // LL corner
            index = Self::LOWER_LEFT;
            canonical_pt = Point::new(
                x - (self.rect.left + self.radii[index].width),
                y - (self.rect.bottom - self.radii[index].height),
            );
            debug_assert!(canonical_pt.x < 0.0 && canonical_pt.y > 0.0);
        } else if x > self.rect.right - self.radii[Self::UPPER_RIGHT].width
            && y < self.rect.top + self.radii[Self::UPPER_RIGHT].height
        {
            // UR corner
            index = Self::UPPER_RIGHT;
            canonical_pt = Point::new(
                x - (self.rect.right - self.radii[index].width),
                y - (self.rect.top + self.radii[index].height),
            );
            debug_assert!(canonical_pt.x > 0.0 && canonical_pt.y < 0.0);
        } else if x > self.rect.right - self.radii[Self::LOWER_RIGHT].width
            && y > self.rect.bottom - self.radii[Self::LOWER_RIGHT].height
        {
            // LR corner
            index = Self::LOWER_RIGHT;
            canonical_pt = Point::new(
                x - (self.rect.right - self.radii[index].width),
                y - (self.rect.bottom - self.radii[index].height),
            );
            debug_assert!(canonical_pt.x > 0.0 && canonical_pt.y > 0.0);
        } else {
            // not in any of the corners
            return true;
        }

        // A point is in an ellipse (in standard position) if:
        //      x^2     y^2
        //     ----- + ----- <= 1
        //      a^2     b^2
        // or :
        //     b^2*x^2 + a^2*y^2 <= (ab)^2
        let dist = canonical_pt.x.square() * self.radii[index].height.square()
            + canonical_pt.y.square() * self.radii[index].width.square();
        dist <= (self.radii[index].width * self.radii[index].height).square()
    }
}

impl From<(Rect, Radius)> for RoundedRect {
    fn from((rect, radius): (Rect, Radius)) -> Self {
        let e = Extent::new(*radius, *radius);
        RoundedRect::from((rect, e))
    }
}

impl From<(Rect, Extent)> for RoundedRect {
    fn from((rect, e): (Rect, Extent)) -> Self {
        RoundedRect::new(&rect, &[e, e, e, e])
    }
}

impl Contains<Point> for RoundedRect {
    fn contains(&self, point: Point) -> bool {
        self.contains(&Rect::new(point, point))
    }
}

impl Contains<&Rect> for RoundedRect {
    fn contains(&self, rect: &Rect) -> bool {
        // Skia: 6d1c0d4196f19537cc64f74bacc7d123de3be454
        if !self.bounds().contains(rect) {
            // If 'rect' isn't contained by the RR's bounds then the
            // RR definitely doesn't contain it
            return false;
        }

        if self.is_rect() {
            // the prior test was sufficient
            return true;
        }

        // At self point we know all four corners of 'rect' are inside the
        // bounds of of self RR. Check to make sure all the corners are inside
        // all the curves
        self.check_corner_containment(self.rect().left, self.rect().top)
            && self.check_corner_containment(self.rect().right, self.rect().top)
            && self.check_corner_containment(self.rect().right, self.rect().bottom)
            && self.check_corner_containment(self.rect().left, self.rect().bottom)
    }
}
