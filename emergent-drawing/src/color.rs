use serde::{Deserialize, Serialize};

// 32-bit ARGB color value.
// TODO: do we really want this? Serialization should be HEX I guess.
// Also: what about a decent color type, say 4 f32 values, may be both?
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Color(pub u32);

impl From<u32> for Color {
    fn from(v: u32) -> Self {
        Color(v)
    }
}
