use crate::Point;
use serde::{Deserialize, Serialize};

/// A line defined by two points.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Line(pub Point, pub Point);
