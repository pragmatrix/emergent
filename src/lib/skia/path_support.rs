use crate::skia::convert::ToSkia;
use crate::PathContainsPoint;
use emergent_drawing::Point;

#[derive(Default)]
pub struct PathSupport {}

impl PathContainsPoint for PathSupport {
    fn path_contains_point(&self, path: &emergent_drawing::Path, p: Point) -> bool {
        path.to_skia().contains(p.to_skia())
    }
}
