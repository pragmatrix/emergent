use crate::{Angle, Matrix, Point, Vector};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Transform {
    Identity,
    Translate(Vector),
    Scale(Vector, Point),
    Rotate(Angle, Point),
    Matrix(Matrix),
}

impl Transform {
    /// Returns a optimized transformation (a * b)
    /// that is equivalent to `Transform::Matrix(Matrix::concat(a.to_matrix(), b.to_matrix()))`
    /// iff an optimization is possible.
    pub fn optimized(a: &Transform, b: &Transform) -> Option<Transform> {
        use Transform::*;
        // TODO: add matrix variants like pre / post_translate().
        match (a, b) {
            (Identity, other) | (other, Identity) => Some(other.clone()),
            (Translate(v1), Translate(v2)) => Some(Translate(*v1 + *v2)),
            (Scale(v1, p1), Scale(v2, p2)) if p1 == p2 => Some(Scale(*v1 * *v2, *p1)),
            (Rotate(a1, p1), Rotate(a2, p2)) if p1 == p2 => Some(Rotate(*a1 + *a2, *p1)),
            (Matrix(m1), Matrix(m2)) => Some(Matrix(crate::Matrix::concat(m1, m2))),
            _ => None,
        }
    }

    pub fn to_matrix(&self) -> Matrix {
        match self {
            Transform::Identity => Matrix::new_identity(),
            Transform::Translate(v) => Matrix::new_translate(*v),
            Transform::Scale(v, p) => Matrix::new_scale(*v, *p),
            Transform::Rotate(degrees, p) => Matrix::new_rotate(*degrees, *p),
            Transform::Matrix(m) => m.clone(),
        }
    }
}
