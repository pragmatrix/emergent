use crate::{FrameLayout, Window};
use emergent_drawing::Point;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
pub use winit::{
    // winit reexports:
    AxisId,
    ElementState,
    KeyboardInput,
    ModifiersState,
    MouseButton,
    MouseScrollDelta,
    TouchPhase,
};

/// The standardized set of messages a Window Application expects from a Windowing system.
///
/// This is modelled after the `WindowEvent` of winit version `0.19.3`.
/// The original `WindowEvent` can not be used because it is not serializable.
/// Some of the variants like `KeyboardInput` do refer public winit types, but these may be ported in the
/// long run.
// TODO: Several of the variants are missing a device identifier because winit
// represents it with a platform dependent type.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum WindowMsg {
    Resized(FrameLayout),
    Moved(Point),
    CloseRequested,
    DroppedFile(PathBuf),
    HoveredFile(PathBuf),
    HoveredFileCancelled,
    ReceivedCharacter(char),
    Focused(bool),
    KeyboardInput(KeyboardInput),
    CursorMoved {
        position: Point,
        modifiers: ModifiersState,
    },
    CursorEntered,
    CursorLeft,
    MouseWheel {
        delta: MouseScrollDelta,
        phase: TouchPhase,
        modifiers: ModifiersState,
    },
    MouseInput {
        state: ElementState,
        button: MouseButton,
        modifiers: ModifiersState,
    },
    TouchpadPressure {
        pressure: f32,
        stage: i64,
    },
    AxisMotion {
        axis: AxisId,
        value: f64,
    },
    Refresh,
    Touch {
        phase: TouchPhase,
        location: Point,
        finger_id: u64,
    },
    HiDPIFactorChanged(FrameLayout),
}

impl WindowMsg {
    /// To create a WindowMsg, we also need some information that can be retrieved from the
    /// Window only.
    pub fn from_window_and_event(
        window: &winit::Window,
        event: winit::WindowEvent,
    ) -> Option<WindowMsg> {
        use winit::WindowEvent::*;

        let to_point = |lp: winit::dpi::LogicalPosition| {
            let pp = lp.to_physical(window.get_hidpi_factor());
            Point::new(pp.x, pp.y)
        };

        match event {
            Resized(_) => Some(WindowMsg::Resized(window.frame_layout())),
            Moved(lp) => Some(WindowMsg::Moved(to_point(lp))),
            CloseRequested => Some(WindowMsg::CloseRequested),
            Destroyed => None,
            DroppedFile(path) => Some(WindowMsg::DroppedFile(path)),
            HoveredFile(path) => Some(WindowMsg::HoveredFile(path)),
            HoveredFileCancelled => Some(WindowMsg::HoveredFileCancelled),
            ReceivedCharacter(c) => Some(WindowMsg::ReceivedCharacter(c)),
            Focused(focused) => Some(WindowMsg::Focused(focused)),
            KeyboardInput {
                device_id: _,
                input,
            } => Some(WindowMsg::KeyboardInput(input)),
            CursorMoved {
                device_id: _,
                position,
                modifiers,
            } => WindowMsg::CursorMoved {
                position: to_point(position),
                modifiers,
            }
            .into(),
            CursorEntered { device_id: _ } => WindowMsg::CursorEntered.into(),
            CursorLeft { device_id: _ } => WindowMsg::CursorLeft.into(),
            MouseWheel {
                device_id: _,
                delta,
                phase,
                modifiers,
            } => WindowMsg::MouseWheel {
                delta,
                phase,
                modifiers,
            }
            .into(),
            MouseInput {
                device_id: _,
                state,
                button,
                modifiers,
            } => Some(WindowMsg::MouseInput {
                state,
                button,
                modifiers,
            }),
            TouchpadPressure {
                device_id: _,
                pressure,
                stage,
            } => Some(WindowMsg::TouchpadPressure { pressure, stage }),
            AxisMotion {
                device_id: _,
                axis,
                value,
            } => Some(WindowMsg::AxisMotion { axis, value }),
            Refresh => Some(WindowMsg::Refresh),
            Touch(winit::Touch {
                device_id: _,
                phase,
                location,
                id,
            }) => Some(WindowMsg::Touch {
                phase,
                location: to_point(location),
                finger_id: id,
            }),
            HiDpiFactorChanged(_) => Some(WindowMsg::HiDPIFactorChanged(window.frame_layout())),
        }
    }

    /// Returns the keyboard modifiers if specified in the Msg, None if not.
    pub fn modifiers(&self) -> Option<ModifiersState> {
        use WindowMsg::*;
        match self {
            KeyboardInput(winit::KeyboardInput { modifiers, .. })
            | CursorMoved { modifiers, .. }
            | MouseWheel { modifiers, .. }
            | MouseInput { modifiers, .. } => Some(*modifiers),
            _ => None,
        }
    }
}
