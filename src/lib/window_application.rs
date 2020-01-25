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

use emergent_drawing::Point;
use emergent_presentation::Presentation;
use emergent_presenter::{AreaHitTest, Host, Support, ViewRenderer};
use emergent_ui::{FrameLayout, ModifiersState, WindowEvent, WindowMessage, WindowState, DPI};
use std::cell::RefCell;
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
    support_builder: Box<dyn Fn(DPI) -> Support>,

    /// The presenter's host.
    ///
    /// TODO: RefCell because we want to use it in tandem with the Model.
    host: RefCell<Host<Msg>>,

    /// State related to the window.
    window_state: WindowState,

    // TODO: do we need a veto-system? Yes, probably, optionally saving state?
    close_requested: bool,

    // A msg to generate when DPIs are changing.
    dpi_msg: Option<Box<dyn Fn(DPI) -> Msg>>,
}

/// A message sent to the window application can be either a `WindowMsg` or
/// an application message.
pub enum WindowApplicationMsg<Msg> {
    Window(WindowEvent),
    Application(Msg),
}

impl<M, Msg> Model<WindowApplicationMsg<Msg>> for WindowApplication<M, Msg>
where
    M: WindowModel<Msg>,
    Msg: Send + 'static,
{
    fn update(&mut self, msg: WindowApplicationMsg<Msg>) -> Cmd<WindowApplicationMsg<Msg>> {
        use WindowApplicationMsg::*;
        match msg {
            Window(msg) => self.update_window(msg),
            Application(msg) => self.update_model(msg),
        }
    }
}

impl<M, Msg> WindowApplication<M, Msg>
where
    M: WindowModel<Msg>,
    Msg: Send + 'static,
{
    pub fn new(
        model: M,
        initial_dpi: DPI,
        support_builder: impl Fn(DPI) -> Support + 'static,
        dpi_msg: Option<impl Fn(DPI) -> Msg + 'static>,
    ) -> Self {
        let support = support_builder(initial_dpi);
        WindowApplication {
            model,
            support_builder: Box::new(support_builder),
            host: Host::new(support).into(),
            window_state: Default::default(),
            close_requested: false,
            dpi_msg: dpi_msg.map(|f| Box::new(f) as Box<dyn Fn(DPI) -> Msg + 'static>),
        }
    }

    pub fn close_requested(&self) -> bool {
        self.close_requested
    }

    fn update_window(&mut self, event: WindowEvent) -> Cmd<WindowApplicationMsg<Msg>> {
        self.window_state.update(event.clone());

        match event {
            WindowEvent::CloseRequested => self.close_requested = true,
            WindowEvent::CursorMoved(_) | WindowEvent::MouseInput { .. } => {
                if let Some(position) = self.window_state.cursor_position() {
                    debug!("position for hit testing {:?}", position);

                    let mut hits = {
                        let host = self.host.borrow();
                        let presentation = host.presentation();
                        presentation.area_hit_test(position, Vec::new(), host.support())
                    };

                    debug!("hits: {:?}", hits);

                    if !hits.is_empty() {
                        let hit = hits.swap_remove(0);
                        let msg = self.host.borrow_mut().dispatch_mouse_input(
                            (hit.0.into(), hit.1),
                            WindowMessage::new(self.window_state.clone(), event),
                        );
                        return msg.map(|msg| self.update_model(msg)).unwrap_or(Cmd::None);
                    }
                }
            }

            WindowEvent::ScaleFactorChanged(frame_layout) => {
                debug!("DPI change: regenerating host");
                let dpi = frame_layout.dpi;
                self.host = Host::new((self.support_builder)(dpi)).into();

                if let Some(dpi_msg) = &self.dpi_msg {
                    let msg = dpi_msg(dpi);
                    return self.update_model(msg);
                }
            }
            _ => {}
        }
        Cmd::None
    }

    fn update_model(&mut self, msg: Msg) -> Cmd<WindowApplicationMsg<Msg>> {
        self.model
            .update(msg)
            .map(WindowApplicationMsg::Application)
    }

    // This function requires self to be mut, because the contained host is modified.
    // TODO: we might consider lying and taking only &self here, because the host is
    //       just a caching mechanism (but also stores the recent presentation).
    pub fn render_presentation(&mut self, frame_layout: &FrameLayout) -> Presentation
    where
        M: ViewRenderer<Msg>,
    {
        self.host
            .borrow_mut()
            .present(frame_layout.clone(), |context| {
                self.model.render_view(context)
            });

        self.host.borrow().presentation().clone()
    }
}

/// In addition to the update function, a model of a window application _may_ implement message
/// filters.
pub trait WindowModel<Msg: Send> {
    fn update(&mut self, msg: Msg) -> Cmd<Msg>;
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
