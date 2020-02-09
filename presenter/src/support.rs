use crate::PathContainsPoint;
use emergent_drawing::{Bounds, MeasureText, Path, Point, Text};
use emergent_ui::DPI;
use std::fmt;
use std::fmt::{Debug, Formatter};

pub struct Support {
    pub dpi: DPI,
    measure: Box<dyn MeasureText>,
    path_contains_point: Box<dyn PathContainsPoint>,
}

impl Debug for Support {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_tuple("dpi").field(&self.dpi).finish()
    }
}

impl Support {
    pub fn new(
        dpi: DPI,
        measure: impl MeasureText + 'static,
        path_contains_point: impl PathContainsPoint + 'static,
    ) -> Self {
        Self {
            dpi,
            measure: Box::new(measure),
            path_contains_point: Box::new(path_contains_point),
        }
    }
}

impl MeasureText for Support {
    fn measure_text(&self, text: &Text) -> Bounds {
        self.measure.measure_text(text)
    }
}

impl PathContainsPoint for Support {
    fn path_contains_point(&self, path: &Path, p: Point) -> bool {
        self.path_contains_point.path_contains_point(path, p)
    }
}
