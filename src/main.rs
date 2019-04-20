use vulkano::sync;
use vulkano::sync::GpuFuture;
use vulkano_win::VkSurfaceBuild;
use winit::{Event, EventsLoop, Window, WindowBuilder, WindowEvent};

mod renderer;
mod skia;

enum WindowStateEvent {
    NoChange,
    /// User wants to close the window.
    CloseRequested,
    /// User resized the window.
    Resized,
}

impl renderer::Window for Window {
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
    let instance = renderer::new_instance();

    let mut events_loop = EventsLoop::new();
    let surface = WindowBuilder::new()
        .build_vk_surface(&events_loop, instance.clone())
        .unwrap();

    let (context, mut frame) = renderer::create_context_and_frame_state(instance, surface);

    let frame = &mut frame;
    let mut future: Box<GpuFuture> = Box::new(sync::now(context.device.clone()));

    loop {
        match process_window_events(&mut events_loop) {
            WindowStateEvent::CloseRequested => return,
            WindowStateEvent::Resized => context.recreate_swapchain(frame),
            WindowStateEvent::NoChange => {}
        }

        future = context.render(future, frame);
    }
}

fn process_window_events(events_loop: &mut EventsLoop) -> WindowStateEvent {
    let mut r = WindowStateEvent::NoChange;

    events_loop.poll_events(|ev| match ev {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } => r = WindowStateEvent::CloseRequested,
        Event::WindowEvent {
            event: WindowEvent::Resized(_),
            ..
        } => r = WindowStateEvent::Resized,
        _ => {}
    });

    r
}
