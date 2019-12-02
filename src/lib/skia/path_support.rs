use crate::skia::convert::ToSkia;
use emergent_drawing::Point;
use emergent_presenter::PathContainsPoint;

#[derive(Default)]
pub struct PathSupport;

impl PathContainsPoint for PathSupport {
    fn path_contains_point(&self, path: &emergent_drawing::Path, p: Point) -> bool {
        path.to_skia().contains(p.to_skia())
    }
}
