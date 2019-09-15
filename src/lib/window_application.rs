//! A Window Application is an application that implements handlers for a a standardized
//! set of messages and implements a number of additional methods to simplify the implementation of
//! applications that expect input from a window and render output, these are:
//!
//! - Hit testing & gesture handling.
//! - Forwarding of messages the application is interested in.
//!
//! A `WindowApplication` is meant to wrap a specific application model implements the `View<Frame>`
//! trait.
//!
//! This can be seen as an intermediate layer that translates messages in and frames out from the
//! core application. If there is a serialization barrier in the application's architecture, it must
//! be between the window application and the renderer.
//!
//! The intended application architecture looks like this:
//!
//! Screen / Window
//!   Renderer
//!     / - Serialization Barrier
//!     | `WindowApplication<WindowMsg<Msg>>`
//!     |   / - Event Sourcing / Simulation Barrier (probably better be placed around the Model?)
//!     |   | `Application<Msg>`
//!     |   |   `Model<Msg>`
//!
//! where messages are sent from top to down, and frames / fender commands from bottom to up.

use crate::{Frame, WindowMsg};
use std::cell::RefCell;
use tears::{Cmd, Model, View};

/// A message sent to the window application can be either a `WindowMsg` or
/// an application message.
pub enum WindowApplicationMsg<Msg> {
    Window(WindowMsg),
    Application(Msg),
}

pub struct WindowApplication<M> {
    /// A copy of the most recent frame, if available.
    recent_frame: RefCell<Option<Frame>>,
    model: M,
}

impl<M, Msg: Send + 'static> Model<WindowApplicationMsg<Msg>> for WindowApplication<M>
where
    M: WindowModel<Msg>,
{
    fn update(&mut self, msg: WindowApplicationMsg<Msg>) -> Cmd<WindowApplicationMsg<Msg>> {
        use WindowApplicationMsg::*;
        if let Some(msg) = self.model.filter_msg(msg) {
            match msg {
                Window(msg) => self.update_window(msg).map(Window),
                Application(msg) => self.update_application(msg).map(Application),
            }
        } else {
            Cmd::None
        }
    }
}

impl<M: View<Frame>> View<Frame> for WindowApplication<M> {
    fn render(&self) -> Frame {
        let frame: Frame = self.model.render();
        // TODO: this is horrible, here the full frame is being cloned.
        // Ideas:
        // - pass the frame back in the WindowMessage that needs it for hit testing.
        // - Use Rc<Frame>, i.e. make it somehow immutable and sharable.
        // - Return only a reference to the internal cache pushing the decision for cloning
        //   to the client but also making the render function implementation dependent, because
        //   it must store the Frame somewhere to return a reference.
        // - Wait until we support incremental presentation
        //   updates and see what other ideas come up.
        self.recent_frame.replace(Some(frame.clone()));
        frame
    }
}

impl<M> WindowApplication<M> {
    pub fn new(model: M) -> Self {
        WindowApplication {
            model,
            recent_frame: RefCell::new(None),
        }
    }

    fn update_window(&mut self, msg: WindowMsg) -> Cmd<WindowMsg> {
        match msg {
            WindowMsg::Resized(_) => {}
            WindowMsg::Moved(_) => {}
            WindowMsg::CloseRequested => {}
            WindowMsg::DroppedFile(_) => {}
            WindowMsg::HoveredFile(_) => {}
            WindowMsg::HoveredFileCancelled => {}
            WindowMsg::ReceivedCharacter(_) => {}
            WindowMsg::Focused(_) => {}
            WindowMsg::KeyboardInput(_) => {}
            WindowMsg::CursorMoved { .. } => {}
            WindowMsg::CursorEntered => {}
            WindowMsg::CursorLeft => {}
            WindowMsg::MouseWheel { .. } => {}
            WindowMsg::MouseInput { .. } => {}
            WindowMsg::TouchpadPressure { .. } => {}
            WindowMsg::AxisMotion { .. } => {}
            WindowMsg::Refresh => {}
            WindowMsg::Touch { .. } => {}
            WindowMsg::HiDPIFactorChanged(_) => {}
        }
        Cmd::None
    }

    fn update_application<Msg: Send>(&mut self, msg: Msg) -> Cmd<Msg>
    where
        M: WindowModel<Msg>,
    {
        self.model.update(msg)
    }
}

/// In addition to the update function, a model of a window application _may_ implement message
/// filters.
pub trait WindowModel<Msg: Send> {
    fn update(&mut self, msg: Msg) -> Cmd<Msg>;

    /// Map a `ApplicationWindowMsg` before it is being processed.
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
