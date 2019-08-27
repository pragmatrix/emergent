use crate::renderer;
use emergent::{AreaLayout, DPI};

impl renderer::Window for winit::Window {
    fn area_layout(&self) -> AreaLayout {
        let dimensions = self
            .get_inner_size()
            .expect("window does not exist anymore");

        let hidpi_factor = self.get_hidpi_factor();
        let dimensions: (u32, u32) = dimensions.to_physical(hidpi_factor).into();
        let dpi = DPI::DEFAULT_SCREEN.map(|dpi| dpi * hidpi_factor);
        AreaLayout { dimensions, dpi }
    }
}
