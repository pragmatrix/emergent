use crate::{Scope, Scoped};
use emergent_drawing::{
    BackToFront, Clip, Clipped, DrawTo, Drawing, DrawingBounds, DrawingFastBounds, DrawingTarget,
    MeasureText, Outset, Paint, ReplaceWith, Transform, Transformed, Visualize, RGB,
};

/// A presentation is a composable hierarchy of drawing commands and scoped areas.
///
/// Scoped areas are used for hit testing and the association of events.
#[derive(Clone, PartialEq, Debug)]
pub enum Presentation {
    Empty,
    /// Defines a presentation scope.
    /// This qualifies all nested names with the scope's name.
    Scoped(Scope, Box<Presentation>),
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

impl Scoped for Presentation {
    fn scoped(self, scope: Scope) -> Self {
        Self::Scoped(scope, self.into())
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

/*
impl<Msg> BackToFront<Presentation<Msg>> for Vec<Presentation<Msg>> {
    fn back_to_front(self) -> Presentation<Msg> {
        Presentation::BackToFront(self.into_iter().collect())
    }
}
*/

impl DrawingFastBounds for Presentation {
    fn fast_bounds(&self, measure: &dyn MeasureText) -> DrawingBounds {
        use Presentation::*;
        match self {
            Empty => DrawingBounds::Empty,
            // note: outset of area is not part of the drawing bounds.
            Scoped(_, nested) | Area(_, nested) => nested.fast_bounds(measure),
            InlineArea(_) => DrawingBounds::Empty,
            Clipped(clip, nested) => nested.fast_bounds(measure).clipped(clip.clone()),
            Transformed(transform, nested) => {
                nested.fast_bounds(measure).transformed(transform.clone())
            }
            BackToFront(nested) => {
                DrawingBounds::union_all(nested.iter().map(|n| n.fast_bounds(measure)))
            }
            Drawing(nested) => nested.fast_bounds(measure),
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

    pub fn scoped(self, scope: impl Into<Scope>) -> Self {
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
