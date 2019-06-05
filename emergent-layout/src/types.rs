use emergent_drawing::scalar;
use std::ops::{Add, Deref, Mul, Sub};

/// A span, a one-dimensional offset and length.
///
/// Note that the offset can be negative.
/// TODO: may a finite scalar type should be introduced.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Span(scalar, fps);
// Scalar can not NaN, so implement Eq
impl Eq for Span {}

pub fn span(start: scalar, dim: impl Into<fps>) -> Span {
    Span(start, dim.into())
}

impl Span {
    pub fn empty() -> Span {
        span(0.0, 0.0)
    }

    pub fn start(&self) -> scalar {
        self.0
    }

    pub fn size(&self) -> fps {
        self.1
    }
}

/// A finite, positive, non-NaN scalar.
///
/// TODO: rename to length().
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct fps(scalar);
impl Eq for fps {}

impl Deref for fps {
    type Target = scalar;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<scalar> for fps {
    fn from(v: scalar) -> Self {
        debug_assert!(v.is_finite() && !v.is_sign_negative());
        fps(v)
    }
}

impl From<usize> for fps {
    fn from(v: usize) -> Self {
        fps::from(v as scalar)
    }
}

impl Add for fps {
    type Output = fps;
    fn add(self, rhs: fps) -> Self::Output {
        self.map(|v| v + rhs.0)
    }
}

impl Sub for fps {
    type Output = fps;
    fn sub(self, rhs: fps) -> Self::Output {
        self.map(|v| v - rhs.0)
    }
}

impl Mul for fps {
    type Output = fps;
    fn mul(self, rhs: fps) -> Self::Output {
        self.map(|v| v * rhs.0)
    }
}

impl fps {
    pub const ZERO: fps = Self(0.0);

    pub fn map<F>(&self, f: F) -> Self
    where
        F: Fn(scalar) -> scalar,
    {
        Self::from(f(self.0))
    }

    pub fn max(&self, other: impl Into<fps>) -> Self {
        let other = other.into();
        self.map(|v| v.max(other.0))
    }

    pub fn min(&self, other: impl Into<fps>) -> Self {
        let other = other.into();
        self.map(|v| v.min(other.0))
    }
}
