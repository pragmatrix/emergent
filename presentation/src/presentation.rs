use crate::{Scope, ScopePath, Scoped};
use emergent_drawing::{
    BackToFront, Bounds, Clip, Clipped, DrawTo, Drawing, DrawingBounds, DrawingFastBounds,
    DrawingTarget, FastBounds, MeasureText, Outset, Paint, ReplaceWith, Transform, Transformed,
    Union, Visualize, RGB,
};
use std::collections::HashSet;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct PresentationMarker;

pub type PresentationScope = Scope<PresentationMarker>;
pub type PresentationPath = ScopePath<PresentationMarker>;

/// A presentation is a composable hierarchy of drawing commands and scoped areas.
///
/// Scoped areas are used for hit testing and the association of events.
#[derive(Clone, PartialEq, Debug)]
pub enum Presentation {
    Empty,
    /// Defines a presentation scope.
    /// This qualifies all nested names with the scope's name.
    Scoped(PresentationScope, Box<Presentation>),
    /// Defines a named area around the (fast) bounds of a presentation, including an Outset.
    Area(Outset, Box<Presentation>),
    /// Defines an area by providing a Clip at the current drawing position and scope.
    InlineArea(Clip),
    /// A clipped presentation (TODO: is that needed, and what exactly is clipped?)
    Clipped(Clip, Box<Presentation>),
    /// A transformed presentation.
    Transformed(Transform, Box<Presentation>),
    /// Multiple presentations, from back to front.
    BackToFront(Vec<Presentation>),
    /// A simple drawing that acts as a presentation.
    Drawing(Drawing),
}

impl Default for Presentation {
    fn default() -> Self {
        Presentation::Empty
    }
}

impl Scoped<PresentationMarker> for Presentation {
    fn scoped(self, scope: impl Into<PresentationScope>) -> Self {
        Self::Scoped(scope.into(), self.into())
    }
}

impl Clipped for Presentation {
    fn clipped(self, clip: impl Into<Clip>) -> Self {
        Self::Clipped(clip.into(), self.into())
    }
}

// Required to support SimpleLayout
impl Transformed for Presentation {
    fn transformed(self, transform: impl Into<Transform>) -> Self {
        Self::Transformed(transform.into(), self.into())
    }
}

impl DrawingFastBounds for Presentation {
    fn fast_bounds(&self, measure: &dyn MeasureText) -> DrawingBounds {
        use Presentation as P;
        match self {
            P::Empty => DrawingBounds::Empty,
            // note: outset of area is not part of the drawing bounds.
            P::Scoped(_, nested) | P::Area(_, nested) => nested.fast_bounds(measure),
            P::InlineArea(_) => DrawingBounds::Empty,
            P::Clipped(clip, nested) => nested.fast_bounds(measure).clipped(clip.clone()),
            P::Transformed(transform, nested) => {
                nested.fast_bounds(measure).transformed(transform.clone())
            }
            P::BackToFront(nested) => {
                DrawingBounds::union_all(nested.iter().map(|n| n.fast_bounds(measure)))
            }
            P::Drawing(nested) => nested.fast_bounds(measure),
        }
    }
}

impl DrawTo for Presentation {
    fn draw_to(&self, current_paint: Paint, target: &mut impl DrawingTarget) {
        use Presentation::*;
        match self {
            Empty => {}
            Scoped(_, nested) | Area(_, nested) => nested.draw_to(current_paint, target),
            InlineArea(_) => {}
            Clipped(clip, nested) => target.clip(clip, |dt| nested.draw_to(current_paint, dt)),
            Transformed(transformed, nested) => {
                target.transform(transformed, |dt| nested.draw_to(current_paint, dt))
            }
            BackToFront(nested) => nested.iter().for_each(|n| n.draw_to(current_paint, target)),
            Drawing(drawing) => drawing.draw_to(current_paint, target),
        }
    }
}

impl Presentation {
    pub fn new() -> Presentation {
        Presentation::Empty
    }

    pub fn in_area(self) -> Self {
        self.in_area_with_outset(Outset::default())
    }

    pub fn in_area_with_outset(self, outset: impl Into<Outset>) -> Self {
        Presentation::Area(outset.into(), self.into())
    }

    pub fn scoped(self, scope: impl Into<PresentationScope>) -> Self {
        Presentation::Scoped(scope.into(), self.into())
    }

    /// Change the presentation so that it provides an open drawing that, when drawn to,
    /// draws above the presentation.
    ///
    /// The opened drawing may not be empty.
    pub fn open_drawing(&mut self) -> &mut Drawing {
        match self {
            Presentation::Empty => {
                self.replace_with(|_| Presentation::Drawing(Drawing::Empty));
                self.open_drawing()
            }
            Presentation::Scoped(_, _)
            | Presentation::Area(_, _)
            | Presentation::InlineArea(_)
            | Presentation::Clipped(_, _)
            | Presentation::Transformed(_, _) => {
                self.replace_with(|p| Presentation::BackToFront(vec![p, Presentation::Empty]));
                self.open_drawing()
            }
            Presentation::BackToFront(presentations) => {
                presentations.last_mut().unwrap().open_drawing()
            }
            Presentation::Drawing(ref mut drawing) => drawing,
        }
    }

    /// Combines this presentation with another one by layering it on top of the current presentation.
    pub fn push_on_top(&mut self, presentation: Presentation) {
        self.replace_with(|p| match p {
            Presentation::Empty => presentation,
            Presentation::BackToFront(mut l) => {
                l.push(presentation);
                Presentation::BackToFront(l)
            }
            Presentation::Scoped(_, _)
            | Presentation::Area(_, _)
            | Presentation::InlineArea(_)
            | Presentation::Clipped(_, _)
            | Presentation::Transformed(_, _)
            | Presentation::Drawing(_) => Presentation::BackToFront(vec![p, presentation]),
        })
    }

    /// Returns a trimmed presentation and its trimmed bounds.
    ///
    /// Trimming a presentation removes elements that are _completely outside_ the given bounds.
    ///
    /// This also means that the returned trimmed size may be larger than the given bounds, e.g. when an
    /// element is partially visible inside the `bounds`.
    pub fn trimmed(self, bounds: Bounds, measure: &dyn MeasureText) -> (Self, DrawingBounds) {
        use Presentation as P;
        match self {
            P::Empty => (self, DrawingBounds::Empty),
            P::Scoped(scope, nested) => {
                let (trimmed, trimmed_bounds) = nested.trimmed(bounds, measure);
                (P::Scoped(scope, trimmed.into()), trimmed_bounds)
            }
            P::Area(outset, nested) => {
                // TODO: handle outset clipping properly.
                let (trimmed, trimmed_bounds) = nested.trimmed(bounds, measure);
                (P::Area(outset, trimmed.into()), trimmed_bounds)
            }
            P::InlineArea(clip) => {
                if let Some(intersection) = Bounds::intersect(&clip.fast_bounds(), &bounds) {
                    // we keep the original clip, but return the intersection with bounds.
                    (P::InlineArea(clip), intersection.into())
                } else {
                    (P::Empty, DrawingBounds::Empty)
                }
            }
            P::Clipped(clip, nested) => {
                let clip_bounds = clip.fast_bounds();
                if let Some(intersection) = Bounds::intersect(&bounds, &clip_bounds) {
                    let (trimmed, trimmed_bounds) = nested.trimmed(intersection, measure);
                    (
                        P::Clipped(clip, trimmed.into()),
                        // trimmed nested bounds may be larger than clip, but they are are expected
                        // to be visually trimmed by the renderer, so we return the intersection
                        // of the clip and the trimmed bounds as the trimmed bounds of the clip.
                        DrawingBounds::intersect(&bounds.into(), &trimmed_bounds),
                    )
                } else {
                    (P::Empty, DrawingBounds::Empty)
                }
            }
            P::Transformed(transform, nested) => {
                if let Some(inv_trans) = transform.invert() {
                    let bounds = bounds.transformed(inv_trans);
                    let (trimmed, trimmed_bounds) = nested.trimmed(bounds, measure);
                    (
                        P::Transformed(transform.clone(), trimmed.into()),
                        trimmed_bounds.transformed(transform),
                    )
                } else {
                    // TODO: log / display an error here.
                    (P::Empty, DrawingBounds::Empty)
                }
            }
            P::BackToFront(mut presentations) => {
                // TODO: remove empty ones?
                let mut trimmed_bounds_union = DrawingBounds::Empty;
                for i in 0..presentations.len() {
                    presentations[i].replace_with(|p| {
                        let (trimmed, trimmed_bounds) = p.trimmed(bounds, measure);
                        trimmed_bounds_union =
                            DrawingBounds::union(trimmed_bounds_union, trimmed_bounds);
                        trimmed
                    });
                }
                (P::BackToFront(presentations), trimmed_bounds_union)
            }
            P::Drawing(ref drawing) => {
                // TODO: may trim drawing's content?
                let drawing_bounds = drawing.fast_bounds(measure);
                if DrawingBounds::intersect(&bounds.into(), &drawing_bounds) != DrawingBounds::Empty
                {
                    (self, drawing_bounds)
                } else {
                    (P::Empty, DrawingBounds::Empty)
                }
            }
        }
    }

    /// Returns all the presentation paths that are used in the presentation.
    pub fn paths(&self) -> HashSet<PresentationPath> {
        // TODO: think about something like a prefix tree for a compact representation of paths.
        let mut set = HashSet::new();
        fill_paths(self, &PresentationPath::new(), &mut set);
        return set;

        fn fill_paths(
            s: &Presentation,
            base: &PresentationPath,
            dict: &mut HashSet<PresentationPath>,
        ) {
            match s {
                Presentation::Scoped(scope, nested) => {
                    let p = base.clone().scoped(scope.clone());
                    fill_paths(nested, &p, dict);
                    dict.insert(p);
                }
                Presentation::Area(_, nested)
                | Presentation::Clipped(_, nested)
                | Presentation::Transformed(_, nested) => fill_paths(nested, base, dict),
                Presentation::BackToFront(presentations) => {
                    presentations.iter().for_each(|p| fill_paths(p, base, dict))
                }
                Presentation::Empty | Presentation::InlineArea(_) | Presentation::Drawing(_) => {}
            }
        }
    }
}

impl Visualize for Presentation {
    fn visualize(&self, measure: &dyn MeasureText) -> Drawing {
        // TODO: const fn!
        // https://www.colorhexa.com/ccff00
        let area_color = 0x00ccff.rgb();
        let clip_color = 0xffcc00.rgb();
        match self {
            Presentation::Empty => Drawing::Empty,
            Presentation::Scoped(_, nested) => nested.visualize(measure),
            Presentation::Area(outset, nested) => {
                // Should we visualize the bounds as an inner rectangle here, too?
                // Bounds + outset could be visualized like that (ascii art of upper left corner only):
                // ____
                // |\__
                // | |

                let nested = nested.visualize(measure);
                let bounds = nested.fast_bounds(measure).outset(outset);
                let bounds_drawing = bounds
                    .visualize(measure)
                    .with_paint(Paint::stroke(area_color));
                [nested, bounds_drawing].to_vec().back_to_front()
            }
            Presentation::InlineArea(clip) => clip
                .visualize(measure)
                .with_paint(Paint::stroke(area_color)),
            Presentation::Clipped(clip, nested) => [
                nested.visualize(measure),
                clip.visualize(measure)
                    .with_paint(Paint::stroke(clip_color)),
            ]
            .to_vec()
            .back_to_front(),
            Presentation::Transformed(t, nested) => {
                nested.visualize(measure).transformed(t.clone())
            }
            Presentation::BackToFront(nested) => nested
                .iter()
                .map(|p| p.visualize(measure))
                .collect::<Vec<Drawing>>()
                .back_to_front(),
            Presentation::Drawing(drawing) => drawing.clone(),
        }
    }
}

pub trait IntoPresentation {
    fn into_presentation(self) -> Presentation;
}

impl IntoPresentation for Drawing {
    fn into_presentation(self) -> Presentation {
        Presentation::Drawing(self)
    }
}

impl From<Drawing> for Presentation {
    fn from(drawing: Drawing) -> Self {
        drawing.into_presentation()
    }
}
