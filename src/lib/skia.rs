use crate::skia::path_support::PathSupport;
use crate::skia::text::PrimitiveText;
use crate::{window_application, DPI};

pub mod convert;
pub mod path_support;
pub mod text;

pub struct Support {
    path: PathSupport,
    measure: PrimitiveText,
}

impl Support {
    pub fn new(dpi: DPI) -> Support {
        Self {
            path: Default::default(),
            measure: PrimitiveText::new(dpi),
        }
    }

    pub fn application(&self) -> window_application::Support {
        window_application::Support::new(&self.measure, &self.path)
    }
}
