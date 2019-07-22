use crate::{Angle, Oval};
use serde::{Deserialize, Serialize};

/// An Arc, described by an oval, start angle, and sweep angle.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Arc(pub Oval, pub Angle, pub Angle);
