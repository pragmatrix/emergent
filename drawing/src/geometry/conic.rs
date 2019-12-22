use crate::{scalar, Point};

/// A conic curve defined by three points and a weight.
#[derive(Clone, PartialEq, Default, Debug)]
pub struct Conic {
    pub points: [Point; 3],
    pub weight: scalar,
}

impl Conic {
    pub fn new(points: &[Point; 3], weight: scalar) -> Self {
        Self {
            points: *points,
            weight,
        }
    }
}

impl From<([Point; 3], scalar)> for Conic {
    fn from((points, weight): ([Point; 3], f64)) -> Self {
        Self { points, weight }
    }
}
