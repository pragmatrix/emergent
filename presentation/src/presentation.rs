use emergent_drawing::{
    BackToFront, Clip, Clipped, DrawTo, Drawing, DrawingBounds, DrawingFastBounds, DrawingTarget,
    MeasureText, Outset, Paint, Transform, Transformed,
};
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Area(&'static str);

impl Area {
    pub const fn new(str: &'static str) -> Area {
        Area(str)
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Debug, Hash)]
pub struct AreaRef(String);

impl From<Area> for AreaRef {
    fn from(area: Area) -> Self {
        AreaRef(area.0.into())
    }
}

/// A presentation is a composable hierarchy that enhances drawings with
/// sensor areas.
#[derive(Clone, PartialEq, Serialize, Deserialize, Debug)]
pub enum Presentation {
    Empty,
    /// Defines a presentation scope.
    /// This qualifies all nested names with the scope's name.
    Scoped(String, Box<Presentation>),
    /// Defines a named area around a presentation, including an Outset.
    Area(AreaRef, Outset, Box<Presentation>),
    /// Defines a named area by providing a Clip at the current drawing position.
    InlineArea(AreaRef, Clip),
    /// A clipped presentation (TODO: is that needed, and what is clipped?)
    Clipped(Clip, Box<Presentation>),
    /// A transformed presentation.
    Transformed(Transform, Box<Presentation>),
    /// Multiple presentations, from back to front.
    BackToFront(Vec<Presentation>),
    /// A simple drawing that acts as a presentation.
    Drawing(Drawing),
}

impl Clipped for Presentation {
    fn clipped(self, clip: Clip) -> Self {
        Presentation::Clipped(clip, self.into())
    }
}

// Required to support SimpleLayout
impl Transformed for Presentation {
    fn transformed(self, transform: Transform) -> Self {
        Presentation::Transformed(transform, self.into())
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
        Presentation::Empty
    }

    pub fn in_area(self, area: Area) -> Self {
        self.in_area_with_outset(area, Outset::default())
    }

    pub fn in_area_with_outset(self, area: Area, outset: impl Into<Outset>) -> Self {
        Presentation::Area(area.into(), outset.into(), self.into())
    }

    pub fn scoped(self, name: impl Into<String>) -> Self {
        Presentation::Scoped(name.into(), self.into())
    }
}
