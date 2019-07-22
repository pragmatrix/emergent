use serde::{Deserialize, Serialize};

/// RGBA color value.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Color(c_comp, c_comp, c_comp, c_comp);

impl Color {
    pub const BLACK: Self = Color(f16::ZERO, f16::ZERO, f16::ZERO, f16::ONE);
    pub const WHITE: Self = Color(f16::ONE, f16::ONE, f16::ONE, f16::ONE);

    /// Convert a color to a u32 bits encoded as ARGB with 8 bit for
    /// each component.
    pub fn to_u32(&self) -> u32 {
        let Color(r, g, b, a) = *self;
        return to_byte(a) << 24 | to_byte(r) << 16 | to_byte(g) << 8 | to_byte(b);

        fn to_byte(v: c_comp) -> u32 {
            (v.to_f32().max(0.0).min(1.0) * 255.0) as u32
        }
    }
}

impl From<u32> for Color {
    /// Convert a u32 ARGB 8 bit component value into a color.
    fn from(v: u32) -> Self {
        let a = v >> 24;
        let r = (v >> 16) & 0xff;
        let g = (v >> 8) & 0xff;
        let b = v & 0xff;
        return Color(from_byte(r), from_byte(g), from_byte(b), from_byte(a));

        fn from_byte(v: u32) -> c_comp {
            c_comp::from_f32(v as f32 / 255.0)
        }
    }
}

impl From<(f32, f32, f32, f32)> for Color {
    /// Convert a color from red, green, blue and alpha values in the range
    /// from 0.0 to 1.0.
    fn from((r, g, b, a): (f32, f32, f32, f32)) -> Self {
        let r = c_comp::from_f32(r);
        let g = c_comp::from_f32(g);
        let b = c_comp::from_f32(b);
        let a = c_comp::from_f32(a);
        Color(r, g, b, a)
    }
}

// A color component.
//
// We use the half crate here and want this to be an implementation detail for now.
#[allow(non_camel_case_types)]
type c_comp = half::f16;

mod f16 {
    use half::consts;

    pub const ZERO: half::f16 = consts::ZERO;
    pub const ONE: half::f16 = consts::ONE;
}
