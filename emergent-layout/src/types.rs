use emergent_drawing::scalar;
use std::ops::{Add, Mul, Sub};

/// A span, an offset and dimension.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Span(fps, fps);

pub fn span(start: impl Into<fps>, dim: impl Into<fps>) -> Span {
    Span(start.into(), dim.into())
}

impl Span {
    pub fn empty() -> Span {
        span(0.0, 0.0)
    }

    pub fn start(&self) -> fps {
        self.0
    }

    pub fn size(&self) -> fps {
        self.1
    }
}

/// A finite, positive, non-NaN scalar with support for Eq.
#[allow(non_upper_case_globals)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct fps(scalar);
impl Eq for fps {}

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
