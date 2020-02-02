use crate::scalar;

pub const SCALAR_1: scalar = 1.0;

pub fn double_to_scalar(s: f64) -> scalar {
    s
}

impl Zero for scalar {
    const ZERO: Self = 0.0;
}

impl SignAsInt for scalar {}

pub trait Zero
where
    Self: Copy,
{
    const ZERO: Self;
}

pub trait SignAsInt: PartialOrd + Zero + Copy {
    fn sign_as_int(&self) -> i32 {
        if *self < Self::ZERO {
            return -1;
        }
        if *self > Self::ZERO {
            return 1;
        }
        0
    }
}
