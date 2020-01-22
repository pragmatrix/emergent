use crate::app::App;
use clap::Arg;
use emergent::skia::convert::ToSkia;
use emergent::skia::path_support::PathSupport;
use emergent::skia::text::PrimitiveText;
use emergent::test_runner::{TestEnvironment, TestRunRequest};
use emergent::{skia, Frame, Msg, WindowApplication, WindowApplicationMsg};
use emergent_config::WindowPlacement;
use emergent_drawing::{font, functions, Font, MeasureText};
use emergent_presenter::Support;
use emergent_ui as ui;
use emergent_ui::{Window, DPI};
use skia_safe::{icu, Typeface};
use std::{env, path, thread};
use tears::{Application, ThreadSpawnExecutor};
use vulkano::sync;
use vulkano::sync::GpuFuture;
use vulkano_win::VkSurfaceBuild;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::desktop::EventLoopExtDesktop;
use winit::window::WindowBuilder;

#[macro_use]
extern crate log;

mod app;
mod renderer;
mod skia_renderer;

fn main() {
    // TODO: push logs internally as soon the window is open?
    env_logger::builder().format_timestamp(None).init();

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

    let mut event_loop = EventLoop::new();

    let initial_window_placement = emergent_config::window_placement::Initial::load();

    let window_surface = {
        let surface = WindowBuilder::new()
            .build_vk_surface(&event_loop, instance.clone())
            .unwrap();
        initial_window_placement.apply_to_window(surface.window());
        surface
    };

    let mut window_placement = WindowPlacement::from_window(window_surface.window())
        .expect("Failed to resolve initial window placement.");
    info!("window placement: {:?}", window_placement);

    let frame_layout = window_surface.window().frame_layout();
    let test_environment = TestEnvironment::new(frame_layout.dpi);
    let test_run_request = TestRunRequest::new_lib(&project_path);
    let (emergent, initial_cmd) = App::new(test_run_request, test_environment);

    info!("spawning application & renderer loop");

    let render_surface = window_surface.clone();
    let mailbox = tears::Mailbox::new();
    let app_mailbox = mailbox.clone();

    let application_thread = thread::spawn(move || {
        let executor = ThreadSpawnExecutor::default();
        let support_builder =
            |dpi: DPI| Support::new(dpi, PrimitiveText::new(dpi), PathSupport::default());
        let mut application = Application::new(
            app_mailbox,
            WindowApplication::new(
                emergent,
                frame_layout.dpi,
                support_builder,
                Some(|dpi| Msg::RerunTestcases(TestEnvironment { dpi })),
            ),
            executor,
        );
        application.schedule(initial_cmd.map(WindowApplicationMsg::Application));
        application.update();

        let (context, mut frame_state) =
            renderer::create_context_and_frame_state(instance, render_surface.clone());
        let frame_state = &mut frame_state;
        let drawing_backend = &mut context.new_skia_backend().unwrap();
        let mut future: Box<dyn GpuFuture> = Box::new(sync::now(context.device.clone()));

        while !application.model().close_requested() {
            let frame_layout = render_surface.window().frame_layout();
            let presentation = application.model_mut().render_presentation(&frame_layout);
            let frame = Frame::new(frame_layout, presentation);

            // even if we drop the frame, we want to recreate the swapchain so that we are
            // prepared for the next (or don't we?).
            if context.need_to_recreate_swapchain(frame_state) {
                context.recreate_swapchain(frame_state)
            }

            let frame_layout = render_surface.window().frame_layout();
            info!("window frame layout: {:?}", frame_layout);
            if frame.layout == frame_layout {
                let _future = context.render(future, frame_state, drawing_backend, &frame);
            } else {
                warn!(
                    "skipping frame, wrong layout, expected {:?}, window: {:?}",
                    frame.layout, frame_layout
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

    event_loop.run_return(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::Resized(_) => {
                    let event =
                        ui::WindowEvent::from_winit(window_surface.window(), event).unwrap();
                    mailbox.post(WindowApplicationMsg::Window(event));
                    // TODO: handle window placement changes in a unified way
                    // (isn't this the application's job?)
                    if window_placement.update(window_surface.window()) {
                        window_placement.store()
                    }
                }
                WindowEvent::Moved(_) => {
                    let event =
                        ui::WindowEvent::from_winit(window_surface.window(), event).unwrap();
                    mailbox.post(WindowApplicationMsg::Window(event));
                    if window_placement.update(window_surface.window()) {
                        window_placement.store()
                    }
                }
                WindowEvent::CloseRequested => {
                    // also forward this to the application, which is expected to shut down in response.
                    let event =
                        ui::WindowEvent::from_winit(window_surface.window(), event).unwrap();
                    mailbox.post(WindowApplicationMsg::Window(event));

                    info!("close requested");
                    *control_flow = ControlFlow::Exit
                }
                event => {
                    if let Some(event) = ui::WindowEvent::from_winit(window_surface.window(), event)
                    {
                        mailbox.post(WindowApplicationMsg::Window(event));
                    }
                }
            },
            event => {
                trace!("unhandled event: {:?}", event);
            }
        }
    });

    info!("events loop out, waiting for application to terminate...");
    application_thread.join().unwrap();
}

// TODO: add a bench for this!
fn _shaper_perf() {
    icu::init();

    let measure = skia::text::PrimitiveText::new(DPI::DEFAULT_SCREEN);

    let font = Font::new("", font::Style::default(), font::Size::new(20.0));
    let text = functions::text("Hello World", &font, None);

    let font = &text.font;
    let typeface =
        Typeface::from_name(&font.name, font.style.to_skia()).expect("failed to resolve typeface");
    let font = skia_safe::Font::from_typeface(&typeface, *font.size as f32);

    for _i in 0..20000 {
        measure.measure_text(&text);
    }
}
