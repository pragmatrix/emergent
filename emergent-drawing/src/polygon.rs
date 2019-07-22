use crate::Point;
use serde::{Deserialize, Serialize};

/// A Polygon, closed when used as a shape, open when added to a path.
// TODO: should a minimum number of ponts be constrained
//       (this is critical for computing bounds())
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Polygon(pub Vec<Point>);
