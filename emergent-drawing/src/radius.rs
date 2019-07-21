use crate::scalar;
use serde::{Deserialize, Serialize};
use std::ops::Deref;

/// Always postive radius.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Radius(scalar);

impl Radius {
    pub fn new(r: scalar) -> Radius {
        assert!(r >= 0.0);
        Radius(r)
    }
}

impl Deref for Radius {
    type Target = scalar;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<scalar> for Radius {
    fn from(radius: scalar) -> Self {
        Radius::new(radius)
    }
}

impl From<i64> for Radius {
    fn from(radius: i64) -> Self {
        (radius as scalar).into()
    }
}
