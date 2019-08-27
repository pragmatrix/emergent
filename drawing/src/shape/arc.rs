use crate::{Angle, Oval};
use serde_tuple::*;

/// An Arc, described by an oval, start angle, and sweep angle.
#[derive(Clone, Serialize_tuple, Deserialize_tuple, PartialEq, Debug)]
pub struct Arc {
    pub oval: Oval,
    pub start: Angle,
    pub sweep: Angle,
}
