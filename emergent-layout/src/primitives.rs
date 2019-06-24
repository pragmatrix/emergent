use emergent_drawing::scalar;
use std::cmp::Ordering;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Deref, DerefMut, Div, Mul, MulAssign, Sub, SubAssign};

/// A finite, non-NaN scalar.
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Default, Debug)]
pub struct finite(scalar);

impl Eq for finite {}

impl Ord for finite {
    fn cmp(&self, other: &Self) -> Ordering {
        (**self).partial_cmp(&**other).unwrap()
    }
}

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

impl AddAssign for finite {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for finite {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        self.map(|v| v - *rhs)
    }
}

impl SubAssign for finite {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul for finite {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        self.map(|v| v * *rhs)
    }
}

impl MulAssign for finite {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Sum for finite {
    fn sum<I: Iterator<Item = finite>>(iter: I) -> Self {
        iter.fold(finite::ZERO, |c, n| c + n)
    }
}

impl finite {
    pub const ZERO: finite = Self(0.0);

    pub fn map<F>(&self, f: F) -> Self
    where
        F: Fn(scalar) -> scalar,
    {
        Self::from(f(**self))
    }

    /*
    pub fn max(&self, other: impl Into<finite>) -> Self {
        let other = other.into();
        self.map(|v| v.max(*other))
    }

    pub fn min(&self, other: impl Into<finite>) -> Self {
        let other = other.into();
        self.map(|v| v.min(*other))
    }*/
}

/// A finite, positive, non-NaN scalar.
#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, PartialOrd, Default, Debug)]
pub struct length(scalar);

impl Eq for length {}

impl Ord for length {
    fn cmp(&self, other: &Self) -> Ordering {
        (**self).partial_cmp(&**other).unwrap()
    }
}

impl Deref for length {
    type Target = scalar;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// TODO: should we support TryFrom?
impl From<scalar> for length {
    fn from(s: scalar) -> Self {
        assert!(s.is_finite() && !s.is_sign_negative());
        length(s)
    }
}

impl From<finite> for length {
    fn from(f: finite) -> Self {
        Self::from(*f)
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
        self.map(|s| s + *rhs)
    }
}

impl AddAssign for length {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Sub for length {
    type Output = length;
    fn sub(self, rhs: length) -> Self::Output {
        self.map(|s| s - *rhs)
    }
}

impl SubAssign for length {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl Mul for length {
    type Output = length;
    fn mul(self, rhs: length) -> Self::Output {
        self.map(|s| s * *rhs)
    }
}

impl MulAssign for length {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}

impl Div for length {
    type Output = length;
    fn div(self, rhs: length) -> Self::Output {
        self.map(|s| s / *rhs)
    }
}

impl Sum for length {
    fn sum<I: Iterator<Item = length>>(iter: I) -> Self {
        iter.fold(length::ZERO, |c, n| c + n)
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

    /*
    pub fn max(&self, other: impl Into<length>) -> Self {
        let other = other.into();
        self.map(|v| v.max(*other))
    }

    pub fn min(&self, other: impl Into<length>) -> Self {
        let other = other.into();
        self.map(|v| v.min(*other))
    }
    */
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
