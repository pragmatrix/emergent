use crate::{scalar, Point};

pub trait SnapToZero {
    fn snap_to_zero(&self, tolerance: f64) -> Self;
}

impl SnapToZero for scalar {
    fn snap_to_zero(&self, tolerance: f64) -> scalar {
        if self.nearly_zero(tolerance) {
            0.0
        } else {
            *self
        }
    }
}

pub trait NearlyZero {
    const NEARLY_ZERO: Self;
    fn nearly_zero(&self, tolerance: scalar) -> bool;
}

impl NearlyZero for scalar {
    // TODO: review, this assumes f32, so can we go as low as 1<<24 for example?
    const NEARLY_ZERO: Self = 1.0 / ((1 << 12) as Self);

    fn nearly_zero(&self, tolerance: f64) -> bool {
        debug_assert!(tolerance >= 0.0);
        self.abs() <= tolerance
    }
}

pub trait NearlyEqual {
    fn nearly_equal(&self, other: &Self, tolerance: scalar) -> bool;
}

impl NearlyEqual for scalar {
    fn nearly_equal(&self, other: &Self, tolerance: scalar) -> bool {
        debug_assert!(tolerance >= 0.0);
        (*self - *other).abs() <= tolerance
    }
}

impl NearlyEqual for Point {
    fn nearly_equal(&self, other: &Self, tolerance: f64) -> bool {
        self.x.nearly_equal(&other.x, tolerance) && self.y.nearly_equal(&other.y, tolerance)
    }
}
