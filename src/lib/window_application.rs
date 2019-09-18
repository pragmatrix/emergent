//! A `WindowApplication` is an application that implements handlers and methods for a standardized
//! set of input messages to simplify applications that expect input from a window and render output,
//! Responsibilities are:
//!
//! - Hit testing & gesture handling.
//! - Forwarding of messages the application is interested in.
//!
//! A `WindowApplication` is meant to wrap a specific application model implements the `View<Frame<Msg>>`
//! trait.
//!
//! This can be seen as an intermediate layer that translates messages in and frames out from the
//! core application. If there is a serialization barrier in the application's architecture, it must
//! be between the renderer and the window application.
//!
//! The intended application architecture looks like this:
//!
//! Screen / Window
//!   Renderer
//!     / - Serialization Barrier
//!     | `WindowApplication<WindowApplicationMsg<Msg>>`
//!     |   / - Event Sourcing / Simulation Barrier (probably better be placed around the Model?)
//!     |   | `Application<Msg>`
//!     |   |   `Model<Msg>`
//!
//! where messages are sent from top to down, and frames / render commands from bottom to up.

use crate::{
    AreaHitTest, ElementState, FrameLayout, ModifiersState, MouseButton, PathContainsPoint,
    RenderPresentation, WindowMsg, DPI,
};
use emergent_drawing::{Bounds, MeasureText, Path, Point, Text};
use emergent_presentation::{Area, DrawingPresentation, Gesture, Presentation};
use std::cell::RefCell;
use tears::{Cmd, Model, View};

/// The generic Window Application Model.
pub struct WindowApplication<Model, Msg>
where
    Model: WindowModel<Msg>,
    Msg: Send + 'static,
{
    /// The actual model of the application.
    model: Model,
    /// A copy of the most recent presentation, if available.
    recent_presentation: RefCell<Option<(DPI, Presentation<Msg>)>>,

    /// State related to input.
    input: InputState,

    /// System support.
    support: Box<dyn Fn(DPI) -> Support>,

    // TODO: do we need a veto-system? Yes, probably, optionally saving state?
    close_requested: bool,
}

/// A message sent to the window application can be either a `WindowMsg` or
/// an application message.
pub enum WindowApplicationMsg<Msg> {
    Window(WindowMsg),
    Application(Msg),
}

impl<M, Msg> Model<WindowApplicationMsg<Msg>> for WindowApplication<M, Msg>
where
    M: WindowModel<Msg>,
    Msg: Send + 'static,
{
    fn update(&mut self, msg: WindowApplicationMsg<Msg>) -> Cmd<WindowApplicationMsg<Msg>> {
        use WindowApplicationMsg::*;
        if let Some(msg) = self.model.filter_msg(msg) {
            match msg {
                Window(msg) => self.update_window(msg),
                Application(msg) => self.update_application(msg),
            }
        } else {
            Cmd::None
        }
    }
}

impl<M, Msg> WindowApplication<M, Msg>
where
    M: WindowModel<Msg>,
    Msg: Send + 'static,
{
    pub fn new(model: M, support_builder: impl Fn(DPI) -> Support + 'static) -> Self {
        WindowApplication {
            recent_presentation: Default::default(),
            model,
            input: Default::default(),
            support: Box::new(support_builder),
            close_requested: false,
        }
    }

    pub fn close_requested(&self) -> bool {
        self.close_requested
    }

    fn update_window(&mut self, msg: WindowMsg) -> Cmd<WindowApplicationMsg<Msg>> {
        match msg {
            WindowMsg::Resized(_) => {}
            WindowMsg::Moved(_) => {}
            WindowMsg::CloseRequested => self.close_requested = true,
            WindowMsg::DroppedFile(_) => {}
            WindowMsg::HoveredFile(_) => {}
            WindowMsg::HoveredFileCancelled => {}
            WindowMsg::ReceivedCharacter(_) => {}
            WindowMsg::Focused(_) => {}
            WindowMsg::KeyboardInput(_) => {}
            WindowMsg::CursorMoved {
                position,
                modifiers,
            } => {
                self.input.cursor = Some(position);
                self.input.modifiers = modifiers
            }
            WindowMsg::CursorEntered => self.input.cursor_entered = true,
            WindowMsg::CursorLeft => self.input.cursor_entered = false,
            WindowMsg::MouseWheel { .. } => {}
            WindowMsg::MouseInput {
                state,
                button,
                modifiers,
            } => {
                if let Some(msg) = {
                    let presentation = &*self.recent_presentation.borrow();
                    if let (Some((dpi, presentation)), Some(position)) =
                        (presentation, self.input.cursor)
                    {
                        // TODO: cache support records.
                        let support = (self.support)(*dpi);
                        presentation
                            .area_hit_test(position, &support)
                            .first()
                            .and_then(|hit| Self::area_mouse_input(*hit, state, button, modifiers))
                    } else {
                        None
                    }
                } {
                    return self.update_application(msg);
                }
            }
            WindowMsg::TouchpadPressure { .. } => {}
            WindowMsg::AxisMotion { .. } => {}
            WindowMsg::Refresh => {}
            WindowMsg::Touch { .. } => {}
            WindowMsg::HiDPIFactorChanged(_) => {}
        }
        Cmd::None
    }

    fn area_mouse_input(
        (area, point): (&Area<Msg>, Point),
        state: ElementState,
        button: MouseButton,
        _modifiers: ModifiersState,
    ) -> Option<Msg> {
        match area {
            Area::Named(name) => {
                debug!("Hit named area: {}", name);
                None
            }
            Area::Gesture(Gesture::Tap(f))
                if state == ElementState::Pressed && button == MouseButton::Left =>
            {
                Some((*f)(point))
            }
            _ => None,
        }
    }

    fn update_application(&mut self, msg: Msg) -> Cmd<WindowApplicationMsg<Msg>> {
        self.model
            .update(msg)
            .map(WindowApplicationMsg::Application)
    }

    pub fn render_presentation(&self, frame_layout: &FrameLayout) -> DrawingPresentation
    where
        M: RenderPresentation<Msg>,
    {
        let presentation: Presentation<Msg> = self.model.render_presentation(frame_layout);
        // TODO: this is horrible, here the full frame is being cloned into the drawing frame.
        // Ideas:
        // - Wait until we support incremental presentation
        //   updates and see what other ideas come up.
        // - Don't clone nested drawings (use Rc?)
        let drawing_presentation = DrawingPresentation::new(&presentation);
        self.recent_presentation
            .replace(Some((frame_layout.dpi, presentation)));
        drawing_presentation
    }
}

/// In addition to the update function, a model of a window application _may_ implement message
/// filters.
pub trait WindowModel<Msg: Send> {
    fn update(&mut self, msg: Msg) -> Cmd<Msg>;

    /// Filter / map a `ApplicationWindowMsg` before it is being processed.
    ///
    /// This can be used to hijack the window input processing logic of the `WindowApplication`.
    ///
    /// The default implementation returns the `ApplicationWindowMsg`.
    fn filter_msg(&self, msg: WindowApplicationMsg<Msg>) -> Option<WindowApplicationMsg<Msg>> {
        use WindowApplicationMsg::*;
        match msg {
            Window(msg) => self.filter_window_msg(msg),
            Application(msg) => self.filter_application_msg(msg),
        }
    }

    fn filter_window_msg(&self, msg: WindowMsg) -> Option<WindowApplicationMsg<Msg>> {
        Some(WindowApplicationMsg::Window(msg))
    }

    fn filter_application_msg(&self, msg: Msg) -> Option<WindowApplicationMsg<Msg>> {
        Some(WindowApplicationMsg::Application(msg))
    }
}

#[derive(Clone, Default, Debug)]
struct InputState {
    /// Current, or most recent cursor position, None if not inside the window.
    cursor: Option<Point>,
    /// Cursor inside the window?
    cursor_entered: bool,
    /// The keyboard modifiers (shift, alt, etc.).
    modifiers: ModifiersState,
}

pub struct Support {
    measure: Box<dyn MeasureText>,
    path_contains_point: Box<dyn PathContainsPoint>,
}

impl Support {
    pub fn new(
        measure: impl MeasureText + 'static,
        path_contains_point: impl PathContainsPoint + 'static,
    ) -> Self {
        Self {
            measure: Box::new(measure),
            path_contains_point: Box::new(path_contains_point),
        }
    }
}

impl MeasureText for Support {
    fn measure_text(&self, text: &Text) -> Bounds {
        self.measure.measure_text(text)
    }
}

impl PathContainsPoint for Support {
    fn path_contains_point(&self, path: &Path, p: Point) -> bool {
        self.path_contains_point.path_contains_point(path, p)
    }
}