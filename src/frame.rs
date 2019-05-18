use emergent_drawing::Painting;

/// A frame represents a sized and layouted, ready to be drawn
/// frame can be rendered to a DrawingTarget.
/// TODO: add some composition here (clipping?) or should we extend drawing to include compositing?
///       What we need are probably some kind of sized areas.
pub struct Frame {
    pub size: (u32, u32),
    pub painting: Painting,
}
