use crate::configuration::Configuration;
use serde::{Deserialize, Serialize};
use std::option::Option;
use std::path::PathBuf;
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use winit::window::Window;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Initial {
    Default,
    Size(LogicalSize<f64>),
    Placement(WindowPlacement),
}

impl Initial {
    /// Loads the initial placement of the window or returns `Initial::Default`.
    pub fn load() -> Initial {
        let placement = WindowPlacement::load();
        match placement {
            Some(placement) => Initial::Placement(placement),
            None => Initial::Default,
        }
    }

    /// Applies the position of the configured window placement to the Window.
    ///
    /// - `default` The default position to use if now configuration wasn't saved yet.
    pub fn apply_to_window(&self, window: &Window) {
        let placement = WindowPlacement::from_initial(*self);

        let pos = placement.and_then(|placement| placement.position());
        let size = placement.and_then(|placement| placement.size());

        // the order in which size and pos is set _is_ significant because
        // if the dpis for some reason.

        if let Some(size) = size {
            window.set_inner_size(size);
        }

        if let Some(pos) = pos {
            window.set_outer_position(pos);
        }

        placement.map(|p| {
            debug!("maximized: {}", p.is_maximized);
            window.set_maximized(p.is_maximized)
        });
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct WindowRect {
    // position is physical (todo: may consider to store logical, so that we place the windows
    // at the same positions even when the scaling changes)
    pub left: f64,
    pub top: f64,
    // size is in logical
    pub width: f64,
    pub height: f64,
}

impl WindowRect {
    pub fn position(&self) -> PhysicalPosition<f64> {
        PhysicalPosition::new(self.left, self.top)
    }

    pub fn size(&self) -> LogicalSize<f64> {
        LogicalSize::new(self.width, self.height)
    }
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct WindowPlacement {
    pub normal: Option<WindowRect>,
    pub maximized: Option<WindowRect>,
    pub is_maximized: bool,
}

impl Configuration for WindowPlacement {
    fn config_path() -> PathBuf {
        "emergent/window-placement".into()
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

    /// Resolves a WindowPlacement from the current window.
    ///
    /// Returns `None` if the `window.state()` returns `WindowState::Minimized`.
    pub fn from_window(window: &Window) -> Option<WindowPlacement> {
        let is_maximized = match window.state() {
            WindowState::Normal => false,
            WindowState::Minimized => return None,
            WindowState::Maximized => true,
        };

        let factor = window.scale_factor();
        let pos = window.outer_position().ok()?;
        let size = window.inner_size().to_logical(factor);

        let rect = WindowRect {
            left: pos.x as _,
            top: pos.y as _,
            width: size.width,
            height: size.height,
        };

        Some(if is_maximized {
            Self {
                normal: None,
                maximized: Some(rect),
                is_maximized: true,
            }
        } else {
            Self {
                normal: Some(rect),
                maximized: None,
                is_maximized: false,
            }
        })
    }

    /// The physical position.
    pub fn position(&self) -> Option<PhysicalPosition<f64>> {
        self.rect().map(|r| r.position())
    }

    /// The logical size.
    pub fn size(&self) -> Option<LogicalSize<f64>> {
        self.rect().map(|r| r.size())
    }

    pub fn rect(&self) -> &Option<WindowRect> {
        if self.is_maximized {
            &self.maximized
        } else {
            &self.normal
        }
    }

    /// Updates the current placement based on the placement of the window.
    /// Returns `true` if the placement changed.
    pub fn update(&mut self, window: &Window) -> bool {
        match Self::from_window(window) {
            Some(placement) if placement.is_maximized => {
                // keep the placement of the non-maximized state.
                if !self.is_maximized {
                    self.maximized = placement.maximized;
                    self.is_maximized = true;
                    true
                } else {
                    if self.maximized != placement.maximized {
                        self.maximized = placement.maximized;
                        true
                    } else {
                        false
                    }
                }
            }
            Some(placement) => {
                if *self != placement {
                    *self = placement;
                    true
                } else {
                    false
                }
            }
            None => false,
        }
    }

    /// Stores the current placement of the Window.
    pub fn store(&self) {
        self.save();
        debug!("Saved window placment: {:?}", self);
    }
}

pub enum WindowState {
    Normal,
    Minimized,
    Maximized,
}

pub trait State {
    fn state(&self) -> WindowState;
}

impl State for Window {
    fn state(&self) -> WindowState {
        let sz = self.inner_size();
        if sz.width == 0 && sz.height == 0 {
            return WindowState::Minimized;
        };

        let id = self.current_monitor();
        let monitor_size = id.size();
        // outer_size is computed slightly too large on Windows.
        let outer_size = self.outer_size();
        debug!("monitor size: {:?}", monitor_size);
        debug!("outer size: {:?}", outer_size);
        // Computation of logical and physical sizes seem to be too large and there is
        // no way to actually find out where the maximizable area is on window (!= screen size).
        // So we'd assume that when one dimension is larger or equal the screen size, the Window
        // is maximized.
        // also: https://github.com/rust-windowing/winit/issues/208
        if outer_size.width >= monitor_size.width || outer_size.height >= monitor_size.height {
            return WindowState::Maximized;
        }
        WindowState::Normal
    }
}

#[cfg(test)]
mod tests {

    use crate::configuration::*;

    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    pub struct WindowPlacement {
        left: i32,
        top: i32,
        width: i32,
        height: i32,
    }

    impl Configuration for WindowPlacement {
        fn config_path() -> PathBuf {
            "tests/window-placement-test".into()
        }
    }

    // Note that tests may run in parallel, so we test this in one test case.
    #[test]
    fn test_config() {
        WindowPlacement::delete();
        assert_eq!(WindowPlacement::load(), None);

        let wl = WindowPlacement {
            left: 101,
            top: 102,
            width: 203,
            height: 304,
        };
        wl.save();

        assert_eq!(WindowPlacement::load(), Some(wl));

        WindowPlacement::delete();
    }
}
