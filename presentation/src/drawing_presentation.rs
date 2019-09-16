use crate::Presentation;
use emergent_drawing::{Clip, DrawTo, Drawing, DrawingTarget, Outset, Paint, Transform};
use serde::{Deserialize, Serialize};

/// A drawing presentation that is not parameterized, but serializable and clonable.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum DrawingPresentation {
    Empty,
    /// Defines a presentation scope.
    /// This qualifies all nested names with the scope's name.
    Scoped(String, Box<DrawingPresentation>),
    /// Defines a named area around the (fast) bounds of a presentation, including an Outset.
    Area(Option<String>, Outset, Box<DrawingPresentation>),
    /// Defines a named area by providing a Clip at the current drawing position.
    InlineArea(Option<String>, Clip),
    /// A clipped presentation (TODO: is that needed, and what exactly is clipped?)
    Clipped(Clip, Box<DrawingPresentation>),
    /// A transformed presentation.
    Transformed(Transform, Box<DrawingPresentation>),
    /// Multiple presentations, from back to front.
    BackToFront(Vec<DrawingPresentation>),
    /// A simple drawing that acts as a presentation.
    Drawing(Drawing),
}

impl DrawingPresentation {
    pub fn new<Msg>(presentation: &Presentation<Msg>) -> DrawingPresentation {
        match presentation {
            Presentation::Empty => DrawingPresentation::Empty,
            Presentation::Scoped(name, nested) => {
                DrawingPresentation::Scoped(name.clone(), Self::new(nested).into())
            }
            Presentation::Area(area, outset, nested) => DrawingPresentation::Area(
                area.name().cloned(),
                outset.clone(),
                Self::new(nested).into(),
            ),
            Presentation::InlineArea(area, clip) => {
                DrawingPresentation::InlineArea(area.name().cloned(), clip.clone())
            }
            Presentation::Clipped(clip, nested) => {
                DrawingPresentation::Clipped(clip.clone(), Self::new(nested).into())
            }
            Presentation::Transformed(transform, nested) => {
                DrawingPresentation::Transformed(transform.clone(), Self::new(nested).into())
            }
            Presentation::BackToFront(presentations) => DrawingPresentation::BackToFront(
                presentations.iter().map(|p| Self::new(p)).collect(),
            ),
            Presentation::Drawing(drawing) => {
                // TODO: copying of drawing isn't what I've expected here, so this should be an Rc
                DrawingPresentation::Drawing(drawing.clone())
            }
        }
    }
}

impl DrawTo for DrawingPresentation {
    fn draw_to(&self, current_paint: Paint, target: &mut impl DrawingTarget) {
        use DrawingPresentation::*;
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
