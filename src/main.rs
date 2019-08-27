use crate::app::App;
use crate::renderer::Window;
use crate::test_runner::TestRunRequest;
use clap::Arg;
use emergent::skia;
use emergent::skia::convert::ToSkia;
use emergent_config::WindowPlacement;
use emergent_drawing::{font, functions, Font, MeasureText};
use skia_safe::{icu, Typeface};
use std::{env, path, thread};
use tears::{Application, ThreadSpawnExecutor, View};
use vulkano::sync;
use vulkano::sync::GpuFuture;
use vulkano_win::VkSurfaceBuild;
use winit::{Event, EventsLoop, WindowBuilder, WindowEvent};

#[macro_use]
extern crate log;

mod app;
mod capture;
mod frame;
mod libtest;
mod renderer;
mod skia_renderer;
mod test_runner;
mod test_watcher;

impl renderer::Window for winit::Window {
    fn physical_size(&self) -> (u32, u32) {
        if let Some(dimensions) = self.get_inner_size() {
            let dimensions: (u32, u32) = dimensions.to_physical(self.get_hidpi_factor()).into();
            dimensions
        } else {
            panic!("window does not exist anymore")
        }
    }
}

fn main() {
    // TODO: push logs internally as soon the window is open?
    env_logger::builder().default_format_timestamp(false).init();

    let matches = clap::App::new("Emergent")
        .author("Armin Sander")
        .about("A visual testrunner for Rust")
        .arg(
            Arg::with_name("PATH")
                .help("The directory of a a Cargo project to run tests and watch for changes.")
                .index(1),
        )
        .get_matches();

    // Init text shaping ICU support.
    icu::init();

    let project_path = {
        let provided = matches.value_of("PATH").map(|p| path::PathBuf::from(p));

        let current_path = path::PathBuf::from(env::current_dir().unwrap());
        provided
            .map(|p| current_path.join(p))
            .unwrap_or(current_path)
    };

    info!("path: {:?}", &project_path);

    let instance = renderer::new_instance();

    let mut events_loop = EventsLoop::new();

    let initial_window_placement = emergent_config::window_placement::Initial::load();

    let window_surface = {
        let default_window_size = winit::dpi::LogicalSize::new(1024.0, 768.0);

        let mut builder = WindowBuilder::new();
        let surface = builder
            .build_vk_surface(&events_loop, instance.clone())
            .unwrap();
        initial_window_placement.apply_to_window(surface.window());
        surface
    };

    let mut window_placement = WindowPlacement::from_window(window_surface.window())
        .expect("Failed to resolve initial window placement.");
    info!("window placement: {:?}", window_placement);

    let test_run_request = TestRunRequest::new_lib(&project_path);
    let window_size = window_surface.window().physical_size();
    let measure = skia::text::SimpleText::new();
    let (emergent, initial_cmd) = App::new(measure, window_size, test_run_request);
    let executor = ThreadSpawnExecutor::default();
    let mut application = Application::new(emergent, executor);
    application.schedule(initial_cmd);

    info!("spawning application & renderer loop");

    // events loop does not implement Send, so we keep it in the main thread, but
    // instead push the renderer loop out.

    let render_surface = window_surface.clone();
    let mailbox = application.mailbox();

    thread::spawn(move || {
        let (context, mut frame_state) =
            renderer::create_context_and_frame_state(instance, render_surface.clone());
        let frame_state = &mut frame_state;
        let drawing_backend = &mut context.new_skia_backend().unwrap();
        let mut future: Box<dyn GpuFuture> = Box::new(sync::now(context.device.clone()));

        loop {
            let frame = application.model().render();

            // even if we drop the frame, we want to recreate the swapchain so that we are
            // prepared for the next (or don't we?).
            if context.need_to_recreate_swapchain(frame_state) {
                context.recreate_swapchain(frame_state)
            }

            let frame_size = frame.size;
            let window_size = render_surface.window().physical_size();
            info!("window size: {:?}", window_size);
            if frame_size == window_size {
                let _future = context.render(future, frame_state, drawing_backend, &frame);
            } else {
                warn!(
                    "skipping frame, wrong size, expected {:?}, window: {:?}",
                    frame_size, window_size
                );
            }

            // if we don't drop the future here, the last image rendered won't be shown.
            // TODO: Can't we do this asynchronously, or run the
            //       application update asynchronously here?
            future = Box::new(sync::now(context.device.clone()));

            application.update();
        }

        debug!("shutting down renderer loop");
    });

    events_loop.run_forever(move |event| match event {
        Event::WindowEvent { event, .. } => match event {
            WindowEvent::Resized(_) => {
                let size = window_surface.window().physical_size();
                mailbox.post(app::Event::WindowResized(size));
                if window_placement.update(window_surface.window()) {
                    window_placement.store()
                }
                winit::ControlFlow::Continue
            }
            WindowEvent::Moved(_) => {
                if window_placement.update(window_surface.window()) {
                    window_placement.store()
                }
                winit::ControlFlow::Continue
            }
            WindowEvent::Refresh => {
                // TODO: not sure if this should go through the app.
                mailbox.post(app::Event::Refresh);
                winit::ControlFlow::Continue
            }
            WindowEvent::CloseRequested => {
                info!("close requested");
                winit::ControlFlow::Break
            }
            WindowEvent::CursorMoved { .. } => winit::ControlFlow::Continue,
            event => {
                debug!("unhandled window event: {:?}", event);
                winit::ControlFlow::Continue
            }
        },
        event => {
            trace!("unhandled event: {:?}", event);
            winit::ControlFlow::Continue
        }
    });

    info!("events loop out");
}

// TODO: add a bench for this!
fn shaper_perf() {
    icu::init();

    let measure = skia::text::SimpleText::new();

    let font = Font::new("", font::Style::default(), font::Size::new(20.0));
    let text = functions::text("Hello World", &font, None);

    let font = &text.font;
    let typeface =
        Typeface::from_name(&font.name, font.style.to_skia()).expect("failed to resolve typeface");
    let font = skia_safe::Font::from_typeface(&typeface, *font.size as f32);

    for i in 0..20000 {
        measure.measure_text(&text);
    }
}
