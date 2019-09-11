use crate::{Path, Rect, RoundedRect, Shape};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Clip {
    Rect(Rect),
    RoundedRect(RoundedRect),
    Path(Path),
}

/// This trait is implemented for types that can represent themselves in a clipped form.
pub trait Clipped {
    fn clipped(self, clip: impl Into<Clip>) -> Self;
}

impl From<Rect> for Clip {
    fn from(r: Rect) -> Self {
        Clip::Rect(r)
    }
}

impl From<RoundedRect> for Clip {
    fn from(rrect: RoundedRect) -> Self {
        Clip::RoundedRect(rrect)
    }
}

impl From<Path> for Clip {
    fn from(path: Path) -> Self {
        Clip::Path(path)
    }
}
