use crate::{Path, Rect, RoundedRect};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Clip {
    Rect(Rect),
    RoundedRect(RoundedRect),
    Path(Path),
}

/// This trait is implemented for types that can represent themselves in a clipped form.
pub trait Clipped {
    fn clipped(self, clip: Clip) -> Self;
}
