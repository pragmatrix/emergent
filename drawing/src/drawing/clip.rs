use crate::{Identity, Path, Rect, RoundedRect};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Clip {
    Unbounded,
    Rect(Rect),
    RoundedRect(RoundedRect),
    Path(Path),
}

impl Identity for Clip {
    /// Identity for Clip in the context of the operations intersect.
    const IDENTITY: Self = Clip::Unbounded;
}
