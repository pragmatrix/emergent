use crate::configuration::Configuration;
use serde::{Deserialize, Serialize};
use std::option::Option;
use std::path::PathBuf;
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize};
use winit::{Window, WindowBuilder};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Initial {
    Default,
    Size(LogicalSize),
    Placement(WindowPlacement),
}

impl Initial {
    /// Loads the initial location of the window or returns `Initial::Default`.
    pub fn load() -> Initial {
        let placement = WindowPlacement::load();
        match placement {
            Some(placement) => Initial::Placement(placement),
            None => Initial::Default,
        }
    }

    /// Loads the initial locations's position and size.
    pub fn load_pos_and_size(default_size: LogicalSize) -> (Option<PhysicalPosition>, LogicalSize) {
        match Self::load() {
            Initial::Default => (None, default_size),
            Initial::Size(size) => (None, size),
            Initial::Placement(placement) => (Some(placement.position()), placement.size()),
        }
    }

    /// Applies the size of the current window location configuration to the WindowBuilder and returns
    /// a new Window Builder instance.
    ///
    /// - `default`: The default size to apply to, if no configuration was found.
    pub fn apply_size(&self, builder: WindowBuilder, default: LogicalSize) -> WindowBuilder {
        let dim = match *self {
            Initial::Default => default,
            Initial::Size(size) => size,
            Initial::Placement(placement) => placement.size(),
        };

        let builder = builder.with_dimensions(dim);

        match *self {
            Initial::Placement(WindowPlacement { is_maximized, .. }) => {
                builder.with_maximized(is_maximized)
            }
            _ => builder,
        }
    }

    /// Apples the position of the configured window location to the Window.
    ///
    /// - `default` The default position to use if now configuration wasn't saved yet.
    pub fn apply_position(&self, window: &Window, default: Option<LogicalPosition>) {
        let dpi_factor = window.get_hidpi_factor();
        let physical_default =
            default.map(|default| PhysicalPosition::from_logical(default, dpi_factor));

        let placement = WindowPlacement::from_initial(*self);

        let pos = placement
            .map(|placement| placement.position())
            .or(physical_default);

        if let Some(pos) = pos {
            let logical = LogicalPosition::from_physical(pos, dpi_factor);
            window.set_position(logical);
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct WindowPlacement {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
    pub is_maximized: bool,
}

impl Configuration for WindowPlacement {
    fn config_path() -> PathBuf {
        "emergent/window-location".into()
    }
}

impl WindowPlacement {
    pub fn from_initial(initial: Initial) -> Option<WindowPlacement> {
        match initial {
            Initial::Default => None,
            Initial::Size(_) => None,
            Initial::Placement(placement) => Some(placement),
        }
    }

    pub fn from_window(window: &Window) -> Option<WindowPlacement> {
        if is_minimized(window) {
            return None;
        };
        let factor = window.get_hidpi_factor();
        let pos = window.get_position().unwrap();
        let physical = PhysicalPosition::from_logical(pos, factor);
        let size = window.get_inner_size().unwrap();

        let assume_is_maximized = {
            let id = window.get_current_monitor();
            let monitor_size = id.get_dimensions();
            // outer_size is computed slightly to large on Windows.
            let monitor_size_logical = LogicalSize::from_physical(monitor_size, factor);
            let outer_size = PhysicalSize::from_logical(window.get_outer_size()?, factor);
            debug!("monitor size: {:?}", monitor_size);
            debug!("monitor size (logical): {:?}", monitor_size_logical);
            debug!("outer size: {:?}", outer_size);
            debug!("outer size (logical): {:?}", window.get_outer_size()?);
            // Computation of logical and physical sizes seem to be too large and there is
            // no way to actually find out where the maximizable area is on window (!= screen size).
            // So we'd assume that when one dimension is larger or equal the screen size, the Window
            // is maximized.
            outer_size.width.round() >= monitor_size.width.round()
                || outer_size.height.round() >= monitor_size.height.round()
        };

        Some(Self {
            left: physical.x,
            top: physical.y,
            width: size.width,
            height: size.height,
            is_maximized: assume_is_maximized,
        })
    }

    /// The physical position.
    pub fn position(&self) -> PhysicalPosition {
        PhysicalPosition::new(self.left, self.top)
    }

    /// The logical size.
    pub fn size(&self) -> LogicalSize {
        LogicalSize::new(self.width, self.height)
    }

    /// Stores the current location and size of the window.
    pub fn store(&self) {
        self.save();
        debug!("Saved window location: {:?}", self);
    }
}

fn is_minimized(window: &Window) -> bool {
    let sz = window.get_inner_size().unwrap();
    sz.width == 0.0 && sz.height == 0.0
}

#[cfg(test)]
mod tests {

    use crate::configuration::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    pub struct WindowLocation {
        left: i32,
        top: i32,
        width: i32,
        height: i32,
    }

    impl Configuration for WindowLocation {
        fn config_path() -> PathBuf {
            "tests/window-location-test".into()
        }
    }

    // Note that tests may run in parallel, so we test this in one test case.
    #[test]
    fn test_config() {
        WindowLocation::delete();
        assert_eq!(WindowLocation::load(), None);

        let wl = WindowLocation {
            left: 101,
            top: 102,
            width: 203,
            height: 304,
        };
        wl.save();

        assert_eq!(WindowLocation::load(), Some(wl));

        WindowLocation::delete();
    }
}
