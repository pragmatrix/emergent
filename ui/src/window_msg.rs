use crate::{FrameLayout, Window};
use emergent_drawing::Point;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use std::time::Instant;
pub use winit::event::{
    // winit re-exports:
    AxisId,
    ElementState,
    KeyboardInput,
    ModifiersState,
    MouseButton,
    MouseScrollDelta,
    TouchPhase,
};

/// A WindowMessage is a WindowEvent annotated with the current state and the time it occurred.
#[derive(Clone, Debug)]
pub struct WindowMessage {
    pub event: WindowEvent,
    pub time: Instant,
    pub state: WindowState,
}

impl WindowMessage {
    pub fn new(event: WindowEvent, time: Instant, state: WindowState) -> Self {
        Self { event, time, state }
    }
}

/// A set of events a window based application expects from a windowing system.
///
/// This is modelled after the `WindowEvent` of winit version `0.19.3`.
///
/// The original `WindowEvent` can not be used because it is not serializable, and we need to support serialization to
/// capture detailed snapshot of input event sequences.
///
/// Some of the variants like `KeyboardInput` do refer public winit types, but these may be ported in the
/// long run.
// TODO: Several of the variants are missing a device identifier because winit
//       represents it with a platform dependent type.
// TODO: make serialization possible again by making Instant serializable if that is even possible.
//       Otherwise replace Instant with something else.

#[derive(Clone, PartialEq, Debug)]
pub enum WindowEvent {
    Resized(FrameLayout),
    Moved(Point),
    CloseRequested,
    DroppedFile(PathBuf),
    HoveredFile(PathBuf),
    HoveredFileCancelled,
    ReceivedCharacter(char),
    Focused(bool),
    KeyboardInput(KeyboardInput),
    CursorMoved(Point),
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
    Tick(Instant),
}

impl WindowEvent {
    /// Import a winit event.
    ///
    /// To create a `WindowEvent`, we also need some information that can be retrieved from the
    /// `Window` only.
    /// TODO: handle DeviceEvent (for modifiers, etc.)
    pub fn from_winit(
        window: &winit::window::Window,
        event: winit::event::WindowEvent,
    ) -> Option<WindowEvent> {
        use winit::event::WindowEvent::*;

        // The converted event should be generated so that by processing them `WindowState` can be derived from.

        match event {
            Resized(_) => Some(WindowEvent::Resized(window.frame_layout())),
            Moved(lp) => Some(WindowEvent::Moved(Point::new(lp.x as _, lp.y as _))),
            CloseRequested => Some(WindowEvent::CloseRequested),
            Destroyed => None,
            DroppedFile(path) => Some(WindowEvent::DroppedFile(path)),
            HoveredFile(path) => Some(WindowEvent::HoveredFile(path)),
            HoveredFileCancelled => Some(WindowEvent::HoveredFileCancelled),
            ReceivedCharacter(c) => Some(WindowEvent::ReceivedCharacter(c)),
            Focused(focused) => WindowEvent::Focused(focused).into(),
            KeyboardInput { input, .. } => Some(WindowEvent::KeyboardInput(input)),
            CursorMoved { position, .. } => {
                let point = Point::new(position.x as _, position.y as _);
                WindowEvent::CursorMoved(point).into()
            }
            CursorEntered { device_id: _ } => WindowEvent::CursorEntered.into(),
            CursorLeft { device_id: _ } => WindowEvent::CursorLeft.into(),
            MouseWheel { delta, phase, .. } => WindowEvent::MouseWheel { delta, phase }.into(),
            MouseInput { state, button, .. } => Some(WindowEvent::MouseInput { state, button }),
            TouchpadPressure {
                pressure, stage, ..
            } => Some(WindowEvent::TouchpadPressure { pressure, stage }),
            AxisMotion { axis, value, .. } => Some(WindowEvent::AxisMotion { axis, value }),
            // TODO: use the force
            Touch(winit::event::Touch {
                phase,
                location,
                id,
                ..
            }) => Some(WindowEvent::Touch {
                phase,
                location: Point::new(location.x as _, location.y as _),
                finger_id: id,
            }),
            ScaleFactorChanged { .. } => {
                Some(WindowEvent::ScaleFactorChanged(window.frame_layout()))
            }
            ThemeChanged(_) => None,
        }
    }

    pub fn left_button_pressed(&self) -> bool {
        match self {
            WindowEvent::MouseInput { state, button, .. }
                if *button == MouseButton::Left && *state == ElementState::Pressed =>
            {
                true
            }
            _ => false,
        }
    }

    pub fn left_button_released(&self) -> bool {
        match self {
            WindowEvent::MouseInput { state, button, .. }
                if *button == MouseButton::Left && *state == ElementState::Released =>
            {
                true
            }
            _ => false,
        }
    }
}

/// We need a custom window state that persists ephemeral information provided with certain events.
#[derive(Clone, Debug, Default)]
pub struct WindowState {
    focused: Option<bool>,
    /// Cursor entered?
    cursor_entered: Option<bool>,
    /// Cursor position, None if not yet set.
    cursor_position: Option<Point>,
    // TODO: add modifiers, cursor_entered, etc.
}

impl WindowState {
    pub fn new(_window: &winit::window::Window) -> WindowState {
        Default::default()
    }

    pub fn update(&mut self, event: WindowEvent) {
        use WindowEvent::*;
        match event {
            Focused(focused) => {
                self.focused = focused.into();
            }
            CursorMoved(position) => {
                self.cursor_position = position.into();
            }
            CursorEntered { .. } => {
                self.cursor_entered = true.into();
            }
            CursorLeft { .. } => {
                self.cursor_entered = false.into();
            }
            _ => (),
        }
    }

    pub fn focused(&self) -> Option<bool> {
        self.focused
    }

    pub fn cursor_position(&self) -> Option<Point> {
        self.cursor_position
    }

    pub fn cursor_entered(&self) -> Option<bool> {
        self.cursor_entered
    }
}

impl WindowEvent {
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
