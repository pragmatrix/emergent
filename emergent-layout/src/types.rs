use emergent_drawing::scalar;
use std::ops::{Add, Deref, Mul, Sub};

/// A span, a one-dimensional offset and length.
///
/// Note that the offset can be negative.
/// TODO: may a finite scalar type should be introduced.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Span(scalar, length);
// Scalar can not NaN, so implement Eq
impl Eq for Span {}

pub fn span(start: scalar, dim: impl Into<length>) -> Span {
    Span(start, dim.into())
}

impl Span {
    pub fn empty() -> Span {
        span(0.0, 0.0)
    }

    pub fn start(&self) -> scalar {
        self.0
    }

    pub fn size(&self) -> length {
        self.1
    }
}

/// A finite, positive, non-NaN scalar.
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct length(scalar);
impl Eq for length {}

impl Deref for length {
    type Target = scalar;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<scalar> for length {
    fn from(v: scalar) -> Self {
        debug_assert!(v.is_finite() && !v.is_sign_negative());
        length(v)
    }
}

impl From<usize> for length {
    fn from(v: usize) -> Self {
        length::from(v as scalar)
    }
}

impl Add for length {
    type Output = length;
    fn add(self, rhs: length) -> Self::Output {
        self.map(|v| v + rhs.0)
    }
}

impl Sub for length {
    type Output = length;
    fn sub(self, rhs: length) -> Self::Output {
        self.map(|v| v - rhs.0)
    }
}

impl Mul for length {
    type Output = length;
    fn mul(self, rhs: length) -> Self::Output {
        self.map(|v| v * rhs.0)
    }
}

impl length {
    pub const ZERO: length = Self(0.0);

    pub fn map<F>(&self, f: F) -> Self
    where
        F: Fn(scalar) -> scalar,
    {
        Self::from(f(self.0))
    }

    pub fn max(&self, other: impl Into<length>) -> Self {
        let other = other.into();
        self.map(|v| v.max(other.0))
    }

    pub fn min(&self, other: impl Into<length>) -> Self {
        let other = other.into();
        self.map(|v| v.min(other.0))
    }
}
