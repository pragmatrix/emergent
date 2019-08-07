use crate::{scalar, Point};
use serde_tuple::*;
use std::ops::Index;

/// A line defined by two points.
#[derive(Clone, Serialize_tuple, Deserialize_tuple, PartialEq, Debug)]
pub struct Line {
    pub point1: Point,
    pub point2: Point,
}

pub fn line(p1: impl Into<Point>, p2: impl Into<Point>) -> Line {
    Line::new(p1.into(), p2.into())
}

pub fn line_v(x: scalar, (y1, y2): (scalar, scalar)) -> Line {
    line((x, y1), (x, y2))
}

pub fn line_h(y: scalar, (x1, x2): (scalar, scalar)) -> Line {
    line((x1, y), (x2, y))
}

impl Line {
    pub const fn new(point1: Point, point2: Point) -> Line {
        Line { point1, point2 }
    }

    pub fn points(&self) -> [Point; 2] {
        [self.point1, self.point2]
    }
}

impl Index<usize> for Line {
    type Output = Point;
    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.point1,
            1 => &self.point2,
            _ => panic!("out of bounds"),
        }
    }
}

impl From<(Point, Point)> for Line {
    fn from((p1, p2): (Point, Point)) -> Self {
        line(p1, p2)
    }
}

impl From<(scalar, scalar, scalar, scalar)> for Line {
    fn from((p1x, p1y, p2x, p2y): (scalar, scalar, scalar, scalar)) -> Self {
        line((p1x, p1y), (p2x, p2y))
    }
}
