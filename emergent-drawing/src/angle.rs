use crate::scalar;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

/// An angle expressed in degrees.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct Angle(scalar);

impl Angle {
    pub fn new(degrees: scalar) -> Angle {
        Angle(degrees)
    }
}

impl Deref for Angle {
    type Target = scalar;
    fn deref(&self) -> &Self::Target {
        &self.0
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
