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
