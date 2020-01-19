use crate::{FrameLayout, Window};
use emergent_drawing::{scalar, Point};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
pub use winit::event::{
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
    },
    CursorEntered,
    CursorLeft,
    MouseWheel {
        delta: MouseScrollDelta,
        phase: TouchPhase,
    },
    MouseInput {
        state: ElementState,
        button: MouseButton,
    },
    TouchpadPressure {
        pressure: f32,
        stage: i64,
    },
    AxisMotion {
        axis: AxisId,
        value: f64,
    },
    Touch {
        phase: TouchPhase,
        location: Point,
        finger_id: u64,
    },
    ScaleFactorChanged(FrameLayout),
}

impl WindowMsg {
    /// To create a WindowMsg, we also need some information that can be retrieved from the
    /// Window only.
    pub fn from_window_and_event(
        window: &winit::window::Window,
        event: winit::event::WindowEvent,
    ) -> Option<WindowMsg> {
        use winit::event::WindowEvent::*;

        let to_point = |lp: winit::dpi::LogicalPosition<scalar>| {
            let pp = lp.to_physical(window.scale_factor());
            Point::new(pp.x, pp.y)
        };

        match event {
            Resized(_) => Some(WindowMsg::Resized(window.frame_layout())),
            Moved(lp) => Some(WindowMsg::Moved(Point::new(lp.x as _, lp.y as _))),
            CloseRequested => Some(WindowMsg::CloseRequested),
            Destroyed => None,
            DroppedFile(path) => Some(WindowMsg::DroppedFile(path)),
            HoveredFile(path) => Some(WindowMsg::HoveredFile(path)),
            HoveredFileCancelled => Some(WindowMsg::HoveredFileCancelled),
            ReceivedCharacter(c) => Some(WindowMsg::ReceivedCharacter(c)),
            Focused(focused) => Some(WindowMsg::Focused(focused)),
            KeyboardInput { input, .. } => Some(WindowMsg::KeyboardInput(input)),
            CursorMoved { position, .. } => WindowMsg::CursorMoved {
                position: Point::new(position.x as _, position.y as _),
            }
            .into(),
            CursorEntered { device_id: _ } => WindowMsg::CursorEntered.into(),
            CursorLeft { device_id: _ } => WindowMsg::CursorLeft.into(),
            MouseWheel { delta, phase, .. } => WindowMsg::MouseWheel { delta, phase }.into(),
            MouseInput { state, button, .. } => Some(WindowMsg::MouseInput { state, button }),
            TouchpadPressure {
                pressure, stage, ..
            } => Some(WindowMsg::TouchpadPressure { pressure, stage }),
            AxisMotion { axis, value, .. } => Some(WindowMsg::AxisMotion { axis, value }),
            // TODO: use the force
            Touch(winit::event::Touch {
                phase,
                location,
                id,
                ..
            }) => Some(WindowMsg::Touch {
                phase,
                location: Point::new(location.x as _, location.y as _),
                finger_id: id,
            }),
            ScaleFactorChanged { .. } => Some(WindowMsg::ScaleFactorChanged(window.frame_layout())),
            ThemeChanged(_) => None,
        }
    }

    /*
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
    */
}
