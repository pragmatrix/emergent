use emergent_drawing::Drawing;

/// A frame is a sized and layouted drawing, ready to be drawn.
#[derive(Clone, PartialEq, Debug)]
pub struct Frame {
    pub area: AreaLayout,
    pub drawing: Drawing,
}

/// The area's layout a frame is drawn to.
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct AreaLayout {
    pub dimensions: (u32, u32),
    pub dpi: f64,
}
