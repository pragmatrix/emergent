use crate::scalar;
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Deref, Sub, SubAssign};

/// An angle expressed in degrees.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Default, Debug)]
pub struct Angle(scalar);

impl Angle {
    pub const ZERO: Angle = Angle(0.0);
    pub const FULL_CIRCLE: Angle = Angle(360.0);

    pub fn new(degrees: scalar) -> Angle {
        Angle(degrees)
    }

    pub fn map(&self, f: impl FnOnce(scalar) -> scalar) -> Angle {
        Angle(f(**self))
    }
}

impl Deref for Angle {
    type Target = scalar;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AddAssign<Angle> for Angle {
    fn add_assign(&mut self, rhs: Angle) {
        *self = *self + rhs;
    }
}

impl Add<Angle> for Angle {
    type Output = Angle;
    fn add(self, rhs: Angle) -> Self::Output {
        self.map(|a| a + *rhs)
    }
}

impl SubAssign<Angle> for Angle {
    fn sub_assign(&mut self, rhs: Angle) {
        *self = *self - rhs;
    }
}

impl Sub<Angle> for Angle {
    type Output = Angle;
    fn sub(self, rhs: Angle) -> Self::Output {
        self.map(|a| a - *rhs)
    }
}

pub trait ToDegrees {
    fn degrees(&self) -> Angle;
}

impl ToDegrees for f64 {
    fn degrees(&self) -> Angle {
        Angle::new(*self)
    }
}
