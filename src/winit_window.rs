use crate::renderer;
use emergent::{FrameLayout, DPI};

impl renderer::Window for winit::Window {
    fn frame_layout(&self) -> FrameLayout {
        let dimensions = self
            .get_inner_size()
            .expect("window does not exist anymore");

        let hidpi_factor = self.get_hidpi_factor();
        let dimensions: (u32, u32) = dimensions.to_physical(hidpi_factor).into();
        let dpi = DPI::DEFAULT_SCREEN.map(|dpi| dpi * hidpi_factor);
        FrameLayout { dimensions, dpi }
    }
}
