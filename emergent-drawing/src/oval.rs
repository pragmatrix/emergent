use crate::Rect;
use serde::{Deserialize, Serialize};

/// An Oval, described by a Rect.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Oval(pub Rect);
