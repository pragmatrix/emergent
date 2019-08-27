mod angle;
pub use angle::*;

pub(crate) mod bounds;
pub use bounds::*;

pub(crate) mod conic;
pub(crate) use conic::*;

pub(crate) mod extent;
pub use extent::*;

mod fast_bounds;
pub use fast_bounds::*;

pub mod matrix;
pub use matrix::Matrix;

pub mod nearly;
pub use nearly::*;

mod outset;
pub use outset::*;

mod radians;
pub use radians::*;

pub(crate) mod radius;
pub use radius::*;

pub(crate) mod vector;
pub use vector::*;

pub trait Union: Sized {
    fn union(this: Self, other: Self) -> Self;

    #[must_use]
    fn union_with(self, other: Self) -> Self {
        Self::union(self, other)
    }
}

impl<T> Union for Option<T>
where
    T: Union,
{
    fn union(this: Self, other: Self) -> Self {
        match (this, other) {
            (None, None) => None,
            (Some(this), None) => Some(this),
            (None, Some(other)) => Some(other),
            (Some(this), Some(other)) => Some(T::union(this, other)),
        }
    }
}
