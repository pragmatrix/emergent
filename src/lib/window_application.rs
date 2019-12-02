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

use crate::RenderPresentation;
use emergent_drawing::{MeasureText, Path, Point, Text};
use emergent_presentation::Presentation;
use emergent_presenter::{Host, Support};
use emergent_ui::{FrameLayout, ModifiersState, WindowMsg, DPI};
use std::cell::RefCell;
use std::marker::PhantomData;
use tears::{Cmd, Model};

/// The generic Window Application Model.
pub struct WindowApplication<Model, Msg>
where
    Model: WindowModel<Msg>,
    Msg: Send + 'static,
{
    /// The actual model of the application.
    model: Model,

    /// System support.
    support: Box<dyn Fn(DPI) -> Support>,

    /// The presenter's host.
    host: RefCell<Option<Host>>,

    /// State related to input.
    input: InputState,

    // TODO: do we need a veto-system? Yes, probably, optionally saving state?
    close_requested: bool,

    msg: PhantomData<Msg>,
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
            model,
            support: Box::new(support_builder),
            host: None.into(),
            input: Default::default(),
            close_requested: false,
            msg: PhantomData,
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
                /*
                if let Some(msg) = {
                    let presentation = &mut *self.recent_presentation.borrow_mut();
                    if let (Some((dpi, presentation)), Some(position)) =
                        (presentation, self.input.cursor)
                    {
                        // TODO: cache support records.
                        let support = (self.support)(*dpi);
                        /*
                        let mut hits = presentation.area_hit_test(position, &support);
                        if !hits.is_empty() {
                            let hit = hits.swap_remove(0);
                            Self::area_mouse_input(hit, state, button, modifiers)
                        } else {
                            None
                        }
                        */
                        None
                    } else {
                        None
                    }
                } {
                return self.update_application(msg);
                } */
                return Cmd::None;
            }
            WindowMsg::TouchpadPressure { .. } => {}
            WindowMsg::AxisMotion { .. } => {}
            WindowMsg::Refresh => {}
            WindowMsg::Touch { .. } => {}
            WindowMsg::HiDPIFactorChanged(_) => {
                // drop the host if the DPI changes.
                self.host = None.into()
            }
        }
        Cmd::None
    }

    /*
    fn area_mouse_input(
        (area, point): (&mut Area<Msg>, Point),
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
                let f = mem::replace(f, Box::new(|_| panic!("event handler already invoked")));
                Some(f(point))
            }
            _ => None,
        }
    }
    */

    fn update_application(&mut self, msg: Msg) -> Cmd<WindowApplicationMsg<Msg>> {
        self.model
            .update(msg)
            .map(WindowApplicationMsg::Application)
    }

    pub fn render_presentation(&self, frame_layout: &FrameLayout) -> Presentation
    where
        M: RenderPresentation<Msg>,
    {
        let mut presentation: Presentation = Presentation::Empty;

        self.host.replace_with(|host| {
            let mut host = host
                .take()
                .unwrap_or_else(|| Host::new((self.support)(frame_layout.dpi)));
            presentation = host
                .present(frame_layout.clone(), |presenter| {
                    // TODO: this is horrible, here the full presentation is being cloned.
                    self.model.render_presentation(presenter);
                })
                .clone();
            Some(host)
        });

        presentation
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
