use crate::{Gesture, Scoped};
use emergent_drawing::{
    BackToFront, Clip, Clipped, DrawTo, Drawing, DrawingBounds, DrawingFastBounds, DrawingTarget,
    MeasureText, Outset, Paint, Render, Transform, Transformed, Visualize, RGB,
};
use serde::{Deserialize, Serialize};

/// A presentation is a composable hierarchy that enhances drawings with
/// sensor areas.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum Presentation {
    Empty,
    /// Defines a presentation scope.
    /// This qualifies all nested names with the scope's name.
    Scoped(String, Box<Presentation>),
    /// Defines a named area around the (fast) bounds of a presentation, including an Outset.
    Area(Area, Outset, Box<Presentation>),
    /// Defines a named area by providing a Clip at the current drawing position.
    InlineArea(Area, Clip),
    /// A clipped presentation (TODO: is that needed, and what exactly is clipped?)
    Clipped(Clip, Box<Presentation>),
    /// A transformed presentation.
    Transformed(Transform, Box<Presentation>),
    /// Multiple presentations, from back to front.
    BackToFront(Vec<Presentation>),
    /// A simple drawing that acts as a presentation.
    Drawing(Drawing),
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
pub enum Area {
    Named(String),
    Gesture(Gesture),
}

impl From<String> for Area {
    fn from(name: String) -> Self {
        Area::Named(name)
    }
}

impl From<Gesture> for Area {
    fn from(gesture: Gesture) -> Self {
        Area::Gesture(gesture)
    }
}

impl Scoped for Presentation {
    fn scoped(self, id: String) -> Self {
        Self::Scoped(id, self.into())
    }
}

impl Clipped for Presentation {
    fn clipped(self, clip: impl Into<Clip>) -> Self {
        Self::Clipped(clip.into(), self.into())
    }
}

// Required to support SimpleLayout
impl Transformed for Presentation {
    fn transformed(self, transform: Transform) -> Self {
        Self::Transformed(transform, self.into())
    }
}

impl BackToFront<Presentation> for Vec<Presentation> {
    fn back_to_front(self) -> Presentation {
        Presentation::BackToFront(self.into_iter().collect())
    }
}

impl DrawingFastBounds for Presentation {
    fn fast_bounds(&self, measure: &dyn MeasureText) -> DrawingBounds {
        use Presentation::*;
        match self {
            Empty => DrawingBounds::Empty,
            // note: outset of area is not part of the drawing bounds.
            Scoped(_, nested) | Area(_, _, nested) => nested.fast_bounds(measure),
            InlineArea(_, _) => DrawingBounds::Empty,
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
            Scoped(_, nested) | Area(_, _, nested) => nested.draw_to(current_paint, target),
            InlineArea(_, _) => {}
            Clipped(clip, nested) => target.clip(clip, |dt| nested.draw_to(current_paint, dt)),
            Transformed(transformed, nested) => {
                target.transform(transformed, |dt| nested.draw_to(current_paint, dt))
            }
            BackToFront(nested) => nested.iter().for_each(|n| n.draw_to(current_paint, target)),
            Drawing(drawing) => drawing.draw_to(current_paint, target),
        }
    }
}

pub trait Present {
    fn present(self) -> Presentation;
}

impl Present for Drawing {
    fn present(self) -> Presentation {
        Presentation::Drawing(self)
    }
}

impl Presentation {
    pub fn new() -> Presentation {
        Self::Empty
    }

    pub fn in_area(self, area: Area) -> Self {
        self.in_area_with_outset(area, Outset::default())
    }

    pub fn in_area_with_outset(self, area: Area, outset: impl Into<Outset>) -> Self {
        Self::Area(area, outset.into(), self.into())
    }

    pub fn scoped(self, name: impl Into<String>) -> Self {
        Self::Scoped(name.into(), self.into())
    }
}

impl Render for Presentation {
    fn render(&self) {}
}

impl Presentation {
    fn visualize(&self, measure: &dyn MeasureText) -> Drawing {
        // TODO: const fn!
        // https://www.colorhexa.com/ccff00
        let area_color = 0x00ccff.rgb();
        let clip_color = 0xffcc00.rgb();
        match self {
            Presentation::Empty => Drawing::Empty,
            Presentation::Scoped(_, nested) => nested.visualize(measure),
            Presentation::Area(_, outset, nested) => {
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
            Presentation::InlineArea(_, clip) => clip
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
