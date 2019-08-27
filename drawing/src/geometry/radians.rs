use crate::{scalar, Angle};
use std::ops::Deref;

#[derive(Copy, Clone, PartialEq, Default, Debug)]
pub struct Radians(scalar);

impl Radians {
    pub const fn new(radians: scalar) -> Self {
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
        Radians::new(d.to_radians())
    }
}
