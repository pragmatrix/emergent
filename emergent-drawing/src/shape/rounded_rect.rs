use crate::{Extent, Point, Radius, Rect, Vector};
use serde::{Deserialize, Serialize};

/// A rounded rectangle.
// TODO: Optimize representation for simple cases?
// Corners are starting at the upper left and follow clockwise.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct RoundedRect(Rect, [Extent; 4]);

impl RoundedRect {
    pub fn new(r: &Rect, radii: &[Extent; 4]) -> Self {
        Self(r.clone(), *radii)
    }

    /// The rect.
    pub fn rect(&self) -> &Rect {
        &self.0
    }

    /// The cordner radii.
    pub fn corner_radii(&self) -> &[Extent; 4] {
        &self.1
    }

    /// Returns corner rectangles in the following order:
    ///
    /// 0-1
    /// | |
    /// 3-2
    pub fn to_corner_rects(&self) -> [Rect; 4] {
        let [r0, r1, r2, r3] = *self.corner_radii();
        [
            Rect::from((self.rect().left_top(), r0.into())),
            Rect::from((
                self.rect().right_top() - Vector::new(r1.width, 0.0),
                r1.into(),
            )),
            Rect::from((self.rect().right_bottom() - r2, r2.into())),
            Rect::from((
                self.rect().left_bottom() - Vector::new(0.0, r3.height),
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
}

impl From<(Rect, Radius)> for RoundedRect {
    fn from((rect, radius): (Rect, Radius)) -> Self {
        let e = Extent::new(*radius, *radius);
        RoundedRect::from((rect, e))
    }
}

impl From<(Rect, Extent)> for RoundedRect {
    fn from((rect, e): (Rect, Extent)) -> Self {
        RoundedRect(rect, [e, e, e, e])
    }
}
