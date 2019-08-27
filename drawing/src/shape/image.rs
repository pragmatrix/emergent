use serde::{Deserialize, Serialize};

// TODO: What should an Image be / refer to? A file, a http:// URL?
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct ImageId(pub String);
