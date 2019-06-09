use crate::Color;

impl From<u32> for Color {
    fn from(v: u32) -> Self {
        Color(v)
    }
}
