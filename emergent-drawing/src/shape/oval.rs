use crate::Rect;
use serde::{Deserialize, Serialize};

/// An Oval, described by a Rect.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Oval(Rect);

impl From<Rect> for Oval {
    fn from(r: Rect) -> Self {
        Oval(r)
    }
}

impl Oval {
    pub fn new(r: &Rect) -> Oval {
        Oval(r.clone())
    }

    pub fn rect(&self) -> &Rect {
        &self.0
    }
}
