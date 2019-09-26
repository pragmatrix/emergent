use emergent_drawing::FromTestEnvironment;
use emergent_presentation::{DrawingPresentation, Presentation};
use emergent_ui::FrameLayout;
use serde::{Deserialize, Serialize};
use std::env;

/// A frame is a sized and layouted drawing, ready to be drawn.
pub struct Frame<Msg> {
    pub layout: FrameLayout,
    pub presentation: Presentation<Msg>,
}

#[derive(Clone, PartialEq, Debug)]
pub struct DrawingFrame {
    pub layout: FrameLayout,
    pub presentation: DrawingPresentation,
}

impl DrawingFrame {
    pub fn new(layout: FrameLayout, presentation: DrawingPresentation) -> DrawingFrame {
        DrawingFrame {
            layout,
            presentation,
        }
    }
}
