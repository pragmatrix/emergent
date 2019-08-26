use crate::configuration::Configuration;
use serde::{Deserialize, Serialize};
use std::option::Option;
use std::path::PathBuf;
use winit::dpi::{LogicalPosition, LogicalSize, PhysicalPosition};
use winit::{Window, WindowBuilder};

pub enum Initial {
    Default,
    Size(LogicalSize),
    PositionAndSize(PhysicalPosition, LogicalSize),
}

impl Initial {
    /// Loads the initial location of the window or returns `Initial::Default`.
    pub fn load() -> Initial {
        let loc = WindowLocation::load();
        match loc {
            Some(loc) => Initial::PositionAndSize(
                PhysicalPosition {
                    x: loc.left,
                    y: loc.top,
                },
                LogicalSize {
                    width: loc.width,
                    height: loc.height,
                },
            ),
            None => Initial::Default,
        }
    }

    /// Loads the initial locations's position and size.
    pub fn load_pos_and_size() -> (Option<PhysicalPosition>, LogicalSize) {
        match Self::load() {
            Initial::Default => (
                None,
                LogicalSize {
                    width: 800.0,
                    height: 600.0,
                },
            ),
            Initial::Size(size) => (None, size),
            Initial::PositionAndSize(pos, size) => (Some(pos), size),
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
            Initial::PositionAndSize(_pos, size) => size,
        };

        builder.with_dimensions(dim)
    }

    /// Apples the position of the configured window location to the Window.
    ///
    /// - `default` The default position to use if now configuration wasn't saved yet.
    pub fn apply_position(&self, window: &Window, default: Option<LogicalPosition>) {
        let dpi_factor = window.get_hidpi_factor();
        let physical_default =
            default.map(|default| PhysicalPosition::from_logical(default, dpi_factor));

        let pos = match *self {
            Initial::Default => physical_default,
            Initial::Size(_) => physical_default,
            Initial::PositionAndSize(pos, _) => Some(pos),
        };

        if let Some(pos) = pos {
            let logical = LogicalPosition::from_physical(pos, dpi_factor);
            window.set_position(logical);
        }
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct WindowLocation {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
}

impl Configuration for WindowLocation {
    fn config_path() -> PathBuf {
        "emergent/window-location".into()
    }
}

impl WindowLocation {
    pub fn from_initial(initial: Initial) -> Option<WindowLocation> {
        match initial {
            Initial::Default => None,
            Initial::Size(_) => None,
            Initial::PositionAndSize(pos, size) => Some(Self {
                left: pos.x,
                top: pos.y,
                width: size.width,
                height: size.height,
            }),
        }
    }

    pub fn from_window(window: &Window) -> Option<WindowLocation> {
        if is_minimized(window) {
            return None;
        };
        let factor = window.get_hidpi_factor();
        let pos = window.get_position().unwrap();
        let physical = PhysicalPosition::from_logical(pos, factor);
        let size = window.get_inner_size().unwrap();
        Some(Self {
            left: physical.x,
            top: physical.y,
            width: size.width,
            height: size.height,
        })
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
