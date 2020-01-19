use crate::{FrameLayout, Window, DPI};

impl Window for winit::window::Window {
    fn frame_layout(&self) -> FrameLayout {
        let dimensions = self.inner_size();

        let scale_factor = self.scale_factor();
        let dimensions = (dimensions.width, dimensions.height);
        let dpi = DPI::DEFAULT_SCREEN.map(|dpi| dpi * scale_factor);

        FrameLayout { dimensions, dpi }
    }
}
