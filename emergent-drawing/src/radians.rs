use crate::{scalar, Angle};
use std::f64::consts::PI;
use std::ops::Deref;

#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct Radians(scalar);

impl Radians {
    pub const fn new(radians: scalar) -> Self {
        // TODO: range-normalize to PI*2?
        Self(radians)
    }
}

impl Deref for Radians {
    type Target = scalar;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Angle> for Radians {
    fn from(d: Angle) -> Self {
        const DEGREE_TO_RADIANS: scalar = PI / 180.0;
        Radians::new(*d * DEGREE_TO_RADIANS)
    }
}
