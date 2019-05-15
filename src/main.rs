use crate::emergent::Emergent;
use crate::renderer::Window;
use crate::test_runner::TestRunRequest;
use core::borrow::Borrow;
use std::{env, thread};
use tears::{Application, ThreadSpawnExecutor, View};
use vulkano::sync;
use vulkano::sync::GpuFuture;
use vulkano_win::VkSurfaceBuild;
use winit::{Event, EventsLoop, WindowBuilder, WindowEvent};

mod capture;
mod emergent;
mod libtest;
mod renderer;
mod skia;
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
    let test_run_request = TestRunRequest::new_lib(&env::current_dir().unwrap());
    let (emergent, initial_cmd) = Emergent::new(test_run_request);
    let executor = ThreadSpawnExecutor::default();
    let (notification_sender, application_notification) = crossbeam_channel::unbounded();
    let notifier = move || notification_sender.send(()).unwrap();
    let mut application = Application::new(emergent, executor, notifier);
    application.schedule(initial_cmd);

    let instance = renderer::new_instance();

    let mut events_loop = EventsLoop::new();
    let window_surface = WindowBuilder::new()
        .build_vk_surface(&events_loop, instance.clone())
        .unwrap();

    dbg!("initial request");

    /*
        // create the initial frame request, so that we have something to show.
        framebuilder.request(window_surface.window().physical_size(), move |frame| {
            renderer_send2
                .send(RendererEvent::RenderFrame(frame))
                .unwrap()
        });
    */

    dbg!("spawning application & renderer");

    // events loop does not implement Send, so we keep it in the main thread, but
    // instead push the renderer loop out.

    let render_surface = window_surface.clone();

    thread::spawn(move || {
        let (context, mut frame_state) =
            renderer::create_context_and_frame_state(instance, render_surface.clone());
        let frame_state = &mut frame_state;
        let drawing_context = &mut context.new_skia_context().unwrap();
        let mut future: Box<GpuFuture> = Box::new(sync::now(context.device.clone()));

        loop {
            application_notification.recv().unwrap();
            let frame = application.model().render();

            // even if we drop the frame, we want to recreate the swapchain so that we are
            // prepared for the next (or don't we?).
            if context.need_to_recreate_swapchain(frame_state) {
                context.recreate_swapchain(frame_state)
            }

            let frame_size = frame.size();
            let window_size = render_surface.window().physical_size();
            if frame_size == window_size {
                future = context.render(future, frame_state, drawing_context, frame.borrow());
            } else {
                println!(
                    "skipping frame, wrong size, expected {:?}, window: {:?}",
                    frame_size, window_size
                );
            }
        }

        dbg!("shutting down renderer loop");
    });

    events_loop.run_forever(move |event| {
        match event {
            Event::WindowEvent {
                event: WindowEvent::Resized(logical_size),
                ..
            } => {
                // resized, compute new physical size and request a new frame.
                println!("window resized, requesting new frame");
                /*
                framebuilder.request(window_surface.window().physical_size(), move |frame| {
                    renderer_send2
                        .send(RendererEvent::RenderFrame(frame))
                        .unwrap()
                }); */
                winit::ControlFlow::Continue
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                dbg!("close requested");
                winit::ControlFlow::Break
            }
            _ => winit::ControlFlow::Continue,
        }
    });

    dbg!("events loop out");
}
