use crate::{Angle, Matrix, Point, Vector};
use serde::{Deserialize, Serialize};
use std::ops::Mul;

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

    pub fn pre_transform(&self, mut m: Matrix) -> Matrix {
        match self {
            Transform::Identity => m.clone(),
            Transform::Translate(d) => {
                m.pre_translate(*d);
                m
            }
            Transform::Scale(v, p) => {
                m.pre_scale(*v, *p);
                m
            }
            Transform::Rotate(angle, p) => {
                m.pre_rotate(*angle, *p);
                m
            }
            Transform::Matrix(tm) => {
                m.pre_concat(tm);
                m
            }
        }
    }
}

impl Mul<Matrix> for &Transform {
    type Output = Matrix;

    fn mul(self, rhs: Matrix) -> Self::Output {
        self.pre_transform(rhs)
    }
}
