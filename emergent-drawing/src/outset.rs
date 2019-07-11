use crate::scalar;
use serde::{Deserialize, Serialize};
use std::ops::Neg;

/// An outset area around a rectangle.
///
/// Previously named Padding, then renamed to precisely specify that
/// - for an actual outset - all values have to be positive. For an inset,
/// all need to be negative.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct Outset(pub [scalar; 4]);

impl Outset {
    pub fn new(left: scalar, top: scalar, right: scalar, bottom: scalar) -> Self {
        [left, top, right, bottom].into()
    }

    pub fn left(&self) -> scalar {
        self.0[0]
    }

    pub fn top(&self) -> scalar {
        self.0[1]
    }

    pub fn right(&self) -> scalar {
        self.0[2]
    }

    pub fn bottom(&self) -> scalar {
        self.0[3]
    }
}

//
// From
//

impl From<scalar> for Outset {
    fn from(outset: scalar) -> Self {
        Self::from((outset, outset))
    }
}

impl From<(scalar, scalar)> for Outset {
    fn from((outset_h, outset_v): (scalar, scalar)) -> Self {
        Self([outset_h, outset_v, outset_h, outset_v])
    }
}

impl From<[scalar; 4]> for Outset {
    fn from(padding: [scalar; 4]) -> Self {
        Self(padding)
    }
}

impl From<(scalar, scalar, scalar, scalar)> for Outset {
    fn from((left, top, right, bottom): (scalar, scalar, scalar, scalar)) -> Self {
        Self::new(left, top, right, bottom)
    }
}

impl Neg for Outset {
    type Output = Self;
    fn neg(self) -> Self {
        Outset::new(-self.left(), -self.top(), -self.right(), -self.bottom())
    }
}
