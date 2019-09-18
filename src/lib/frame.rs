use emergent_drawing::FromTestEnvironment;
use emergent_presentation::{DrawingPresentation, Presentation};
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

/// The frame's expected layout expressed in physical pixel dimensions and
/// dots per inch.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct FrameLayout {
    pub dimensions: (u32, u32),
    pub dpi: DPI,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct DPI(pub f64);

impl DPI {
    /// The default DPI assumed for a HiDPI scaling factor of 1.
    // https://github.com/rust-windowing/winit/issues/920
    pub const DEFAULT_SCREEN: DPI = Self::new(96.0);
    pub const DEFAULT_POINTS: DPI = Self::new(72.0);

    pub const fn new(dpi: f64) -> Self {
        Self(dpi)
    }

    /// Assuming `self` represents screen DPIs, this scales font points to the pixel
    /// resolution of the screen.
    pub fn scale_font_points(&self, points: f64) -> f64 {
        points * self.0 / Self::DEFAULT_POINTS.0
    }

    pub fn map(&self, f: impl FnOnce(f64) -> f64) -> DPI {
        DPI(f(self.0))
    }
}

impl FromTestEnvironment for DPI {
    fn from_test_environment() -> Self {
        env::var("EMERGENT_TEST_DPI")
            .ok()
            .and_then(|v| v.parse().ok())
            .map(DPI::new)
            .unwrap_or(DPI::DEFAULT_SCREEN)
    }
}
