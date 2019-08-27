use crate::Point;
use serde::{Deserialize, Serialize};

/// A Polygon, closed when used as a shape, open when added to a path.
// TODO: should a minimum number of ponts be constrained
//       (this is critical for computing bounds())
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Polygon(Vec<Point>);

impl Polygon {
    pub fn points(&self) -> &[Point] {
        &self.0
    }
}

impl<I: IntoIterator<Item = Point>> From<I> for Polygon {
    fn from(i: I) -> Self {
        Self(i.into_iter().collect())
    }
}
