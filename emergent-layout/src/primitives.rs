use emergent_drawing::scalar;
use std::ops::{Add, Deref, Mul, Sub};

/// A finite, non-NaN scalar.
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct finite(scalar);
impl Eq for finite {}

impl Deref for finite {
    type Target = scalar;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<scalar> for finite {
    fn from(v: scalar) -> Self {
        assert!(v.is_finite());
        finite(v)
    }
}

impl From<isize> for finite {
    fn from(v: isize) -> Self {
        finite::from(v as scalar)
    }
}

impl Add for finite {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        self.map(|v| v + *rhs)
    }
}

impl Sub for finite {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self.map(|v| v - *rhs)
    }
}

impl Mul for finite {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        self.map(|v| v * *rhs)
    }
}

impl finite {
    pub const ZERO: finite = Self(0.0);

    pub fn map<F>(&self, f: F) -> Self
    where
        F: Fn(scalar) -> scalar,
    {
        Self::from(f(self.0))
    }

    pub fn max(&self, other: impl Into<finite>) -> Self {
        let other = other.into();
        self.map(|v| v.max(other.0))
    }

    pub fn min(&self, other: impl Into<finite>) -> Self {
        let other = other.into();
        self.map(|v| v.min(other.0))
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
        assert!(v.is_finite() && !v.is_sign_negative());
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

//
// finite <-> length
//

impl Add<length> for finite {
    type Output = Self;
    fn add(self, rhs: length) -> Self::Output {
        self.map(|v| v + *rhs)
    }
}

impl Sub<length> for finite {
    type Output = Self;
    fn sub(self, rhs: length) -> Self::Output {
        self.map(|v| v - *rhs)
    }
}
