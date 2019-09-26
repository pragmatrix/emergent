use emergent_presentation::Presentation;
use emergent_ui::FrameLayout;

/// A frame is a sized and layouted drawing, ready to be drawn.
#[derive(Clone, Debug)]
pub struct Frame {
    pub layout: FrameLayout,
    pub presentation: Presentation,
}

impl Frame {
    pub fn new(layout: FrameLayout, presentation: Presentation) -> Self {
        Self {
            layout,
            presentation,
        }
    }
}
