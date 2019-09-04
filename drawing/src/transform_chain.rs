//! A transform chain is a directed acyclic graph of transforms applied after another to
//! form a transformation hierarchy.

use crate::{lazy_map, Chain, Identity, Matrix, Ref, Transform};

impl Identity for Transform {
    const IDENTITY: Self = Transform::Identity;
}

impl Identity for Matrix {
    const IDENTITY: Self = Matrix::IDENTITY;
}

impl dyn Chain<Transform> {
    pub fn matrices<'a>(&'a self) -> impl Chain<Matrix> + 'a {
        fn map(m: &Matrix, transform: &Transform) -> Matrix {
            transform.pre_transform(m.clone())
        };

        lazy_map(self, map)
    }
}
