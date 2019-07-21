use crate::scalar;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Degrees(scalar);

impl Degrees {
    pub fn new(degrees: scalar) -> Degrees {
        Degrees(degrees)
    }
}

impl Deref for Degrees {
    type Target = scalar;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait ToDegrees {
    fn degrees(&self) -> Degrees;
}

impl ToDegrees for f64 {
    fn degrees(&self) -> Degrees {
        Degrees::new(*self)
    }
}
