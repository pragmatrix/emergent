use crate::{Path, Rect, RoundedRect};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Clip {
    Rect(Rect),
    RoundedRect(RoundedRect),
    Path(Path),
}
