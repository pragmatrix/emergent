use crate::{scalar, Extent, Point, Rect, Scalar, Vector};
use serde_tuple::*;

/// A rounded rectangle.
// TODO: Optimize representation for simple cases?
// Corners are starting at the upper left and follow clockwise.
#[derive(Clone, Serialize_tuple, Deserialize_tuple, PartialEq, Debug)]
pub struct RoundedRect {
    rect: Rect,
    // TODO: replace this with Radii?
    radii: [Extent; 4],
}

pub fn rounded_rect(rect: impl Into<Rect>, radii: impl Into<internal::Radii>) -> RoundedRect {
    RoundedRect::new(rect.into(), radii.into().0)
}

impl RoundedRect {
    pub fn new(rect: Rect, radii: [Extent; 4]) -> Self {
        Self { rect, radii }
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

pub mod traits {
    use crate::{Extent, Radius, Rect, RoundedRect};

    impl From<(Rect, Radius)> for RoundedRect {
        fn from((rect, radius): (Rect, Radius)) -> Self {
            let e = Extent::new(*radius, *radius);
            RoundedRect::from((rect, e))
        }
    }

    impl From<(Rect, Extent)> for RoundedRect {
        fn from((rect, e): (Rect, Extent)) -> Self {
            RoundedRect::new(rect, [e, e, e, e])
        }
    }
}

pub mod contains {
    use crate::{Contains, Point, Rect, RoundedRect};

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
            self.check_corner_containment(rect.left, rect.top)
                && self.check_corner_containment(rect.right, rect.top)
                && self.check_corner_containment(rect.right, rect.bottom)
                && self.check_corner_containment(rect.left, rect.bottom)
        }
    }
}

pub mod internal {
    use crate::{scalar, Extent};

    #[derive(Copy, Clone, PartialEq, Debug)]
    pub struct Radii(pub(crate) [Extent; 4]);

    impl From<(i32, i32)> for Radii {
        fn from((w, h): (i32, i32)) -> Self {
            Radii::from((w as scalar, h as scalar))
        }
    }

    impl From<(scalar, scalar)> for Radii {
        fn from((w, h): (scalar, scalar)) -> Self {
            Radii::from(Extent::new(w, h))
        }
    }

    impl From<Extent> for Radii {
        fn from(x: Extent) -> Self {
            (x, x).into()
        }
    }

    impl From<(Extent, Extent)> for Radii {
        fn from((top, bottom): (Extent, Extent)) -> Self {
            [top, top, bottom, bottom].into()
        }
    }

    impl From<[Extent; 4]> for Radii {
        fn from(x: [Extent; 4]) -> Self {
            Radii(x)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::functions::{rect, rounded_rect};
    use crate::{Color, Contains, Drawing, DrawingTarget, Paint, Point, Render, RGB};
    use rand::rngs::StdRng;
    use rand::{Rng, SeedableRng};

    #[test]
    pub fn contains() {
        let mut drawing = Drawing::new();

        let rr = rounded_rect(rect((16.0, 16.0), (64, 64)), (16.0, 16.0));
        drawing.draw(rr.bounds().clone(), Paint::stroke(0xc0c0c0.rgb()));
        drawing.draw(rr.clone(), Paint::stroke(Color::BLACK));

        let mut rng: StdRng = SeedableRng::seed_from_u64(0);
        let hit_color = 0x00ff00.rgb();
        let missed_color = 0xff0000.rgb();

        for _ in 0..128 {
            let x = rng.gen_range(0, 96);
            let y = rng.gen_range(0, 96);
            let p: Point = (x, y).into();
            let color = if rr.contains(p) {
                hit_color
            } else {
                missed_color
            };
            drawing.draw(rect(p, (2, 2)), color)
        }

        drawing.render()
    }
}
