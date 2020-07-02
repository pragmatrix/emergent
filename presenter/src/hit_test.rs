//! Single point presentation hit testing.

use crate::Support;
use emergent_drawing::{Clip, Contains, DrawingFastBounds, Path, Point};
use emergent_presentation::{Presentation, PresentationPath};

pub trait PathContainsPoint {
    fn path_contains_point(&self, path: &Path, p: Point) -> bool;
}

pub trait HitTest {
    fn hit_test(&self, p: Point, path_tester: &dyn PathContainsPoint) -> bool;
}

pub trait AreaHitTest {
    /// Hit tests at `p` and returns hits from back to front.
    ///
    /// For each hit, returns a tuple `PresentationPath` & `Point`, where the path describes
    /// where the hit took place and `Point` describes the hit point relative to the coordinate
    /// system the area was placed.
    ///
    /// TODO: The `Point` returned is relative to the area it hit. But it should be relative
    ///       to the scope that surrounds the area it hit? And also the Scope's area should probably
    ///       by returned.
    fn area_hit_test(
        &self,
        p: Point,
        scope: PresentationPath,
        support: &dyn Support,
    ) -> Vec<(PresentationPath, Point)>;
}

impl AreaHitTest for Presentation {
    fn area_hit_test(
        &self,
        p: Point,
        mut scope: PresentationPath,
        support: &dyn Support,
    ) -> Vec<(PresentationPath, Point)> {
        match self {
            Presentation::Empty => Vec::new(),
            Presentation::Scoped(s, nested) => {
                // TODO: avoid the clone here? Can't we generate these scope paths a bit more gently?
                scope.push(s.clone());
                nested.area_hit_test(p, scope, support)
            }
            Presentation::Area(outset, nested) => {
                let nested_bounds_plus_outset =
                    nested.fast_bounds(support.measure_text()).outset(outset);
                // TODO: scope gets cloned here!
                let mut hits = nested.area_hit_test(p, scope.clone(), support);
                if nested_bounds_plus_outset.contains(p) {
                    hits.push((scope, p))
                }
                hits
            }
            Presentation::InlineArea(clip) => {
                if clip.hit_test(p, support.path_contains_point()) {
                    vec![(scope, p)]
                } else {
                    Vec::new()
                }
            }
            Presentation::Clipped(clip, nested) => {
                // clip clips both, areas and drawings for now.
                if clip.hit_test(p, support.path_contains_point()) {
                    nested.area_hit_test(p, scope, support)
                } else {
                    Vec::new()
                }
            }
            Presentation::Transformed(t, nested) => {
                let p = t.invert().unwrap().map_point(p);
                nested.area_hit_test(p, scope, support)
            }
            Presentation::BackToFront(nested) => {
                nested.iter().fold(Vec::new(), |mut c, nested| {
                    // TODO: avoid that scope.clone() here.
                    c.extend(nested.area_hit_test(p, scope.clone(), support));
                    c
                })
            }
            Presentation::Drawing(_) => Vec::new(),
        }
    }
}

impl HitTest for Clip {
    fn hit_test(&self, p: Point, support: &dyn PathContainsPoint) -> bool {
        match self {
            Clip::Rect(r) => r.contains(p),
            Clip::RoundedRect(rr) => rr.contains(p),
            Clip::Path(path) => support.path_contains_point(path, p),
        }
    }
}
