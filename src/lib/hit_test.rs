//! Single point presentation hit testing.

use emergent_drawing::{Clip, Contains, DrawingFastBounds, MeasureText, Path, Point};
use emergent_presentation::{Area, Presentation};

pub trait PathContainsPoint {
    fn path_contains_point(&self, path: &Path, p: Point) -> bool;
}

pub trait HitTest {
    fn hit_test(&self, p: Point, path_tester: &dyn PathContainsPoint) -> bool;
}

pub trait AreaHitTest<Msg> {
    /// Hit tests at the given point and returns a vector of areas from back to front being the
    /// last record to describe the frontmost positive test.
    /// Returns a tuple Area & Point, where Point describes the hit point relative to the
    /// coordinate system the area was placed.
    // TODO: don't require Path hit testing to be injected.
    // TODO: return named scopes?
    fn area_hit_test(
        &self,
        p: Point,
        support: &(impl PathContainsPoint + MeasureText),
    ) -> Vec<(&Area<Msg>, Point)>;
}

impl<Msg> AreaHitTest<Msg> for Presentation<Msg> {
    fn area_hit_test(
        &self,
        p: Point,
        support: &(impl PathContainsPoint + MeasureText),
    ) -> Vec<(&Area<Msg>, Point)> {
        match self {
            Presentation::Empty => Vec::new(),
            Presentation::Scoped(_, nested) => nested.area_hit_test(p, support),
            Presentation::Area(area, outset, presentation) => {
                let mut nested = presentation.area_hit_test(p, support);
                let nested_bounds_plus_outset = presentation.fast_bounds(support).outset(outset);
                if nested_bounds_plus_outset.contains(p) {
                    nested.push((area, p))
                }
                nested
            }
            Presentation::InlineArea(area, clip) => {
                if clip.hit_test(p, support) {
                    vec![(area, p)]
                } else {
                    Vec::new()
                }
            }
            Presentation::Clipped(clip, presentation) => {
                // clip clips both, areas and drawings for now.
                if clip.hit_test(p, support) {
                    presentation.area_hit_test(p, support)
                } else {
                    Vec::new()
                }
            }
            Presentation::Transformed(t, presentation) => {
                let p = t.invert().unwrap().map_point(p);
                presentation.area_hit_test(p, support)
            }
            Presentation::BackToFront(presentations) => {
                // this is particularly inefficient generating these vectors in the first place.
                // probably we need to pass some array nested hit tests can add to.
                presentations
                    .iter()
                    .fold(Vec::new(), |mut c, presentation| {
                        c.extend(presentation.area_hit_test(p, support));
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
