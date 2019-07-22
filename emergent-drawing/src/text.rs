use crate::{Font, Point};
use serde::{Deserialize, Serialize};

/// Text, described by a location, a string, and the font.
// TODO: can we share fonts?
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Text(pub Point, pub String, pub Font);
