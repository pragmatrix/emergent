use std::sync::Arc;
use vulkano::buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass};
use vulkano::image::SwapchainImage;
use vulkano::instance::{Instance, PhysicalDevice};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::swapchain;
use vulkano::swapchain::{
    PresentMode, Surface, SurfaceTransform, Swapchain, SwapchainCreationError,
};
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};

#[derive(Debug, Clone)]
struct Vertex {
    position: [f32; 2],
}

pub struct RenderContext<W: Window> {
    surface: Arc<Surface<W>>,
    device: Arc<Device>,
    queue: Arc<Queue>,
    vertex_buffer: Vec<Arc<BufferAccess + Send + Sync>>,
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
}

pub struct FrameState<W: Window> {
    dynamic_state: DynamicState,
    recreate_swapchain: bool,
    previous_frame_end: Option<Box<GpuFuture>>,
    swapchain: Arc<Swapchain<W>>,
    framebuffers: Vec<Arc<FramebufferAbstract + Send + Sync>>,
}

pub trait Window: Send + Sync + 'static {
    fn physical_size(&self) -> (u32, u32);
}

pub fn new_instance() -> Arc<Instance> {
    let extensions = vulkano_win::required_extensions();
    Instance::new(None, &extensions, None).unwrap()
}

pub fn create_context_and_frame_state<W: Window>(
    instance: Arc<Instance>,
    surface: Arc<Surface<W>>,
) -> (RenderContext<W>, FrameState<W>) {
    // TODO: select a proper physical device, the first one might not be suitable for
    //       rendering on the screen.
    let physical = PhysicalDevice::enumerate(&instance).next().unwrap();
    println!(
        "Using device: {} (type: {:?})",
        physical.name(),
        physical.ty()
    );

    let queue_family = physical
        .queue_families()
        .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
        .unwrap();

    let device_ext = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::none()
    };
    let (device, mut queues) = Device::new(
        physical,
        physical.supported_features(),
        &device_ext,
        [(queue_family, 0.5)].iter().cloned(),
    )
    .unwrap();

    let queue = queues.next().unwrap();

    let (swapchain, images) = {
        let caps = surface.capabilities(physical).unwrap();

        let usage = caps.supported_usage_flags;

        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;

        let window = surface.window();
        let initial_dimensions = window.physical_size();

        Swapchain::new(
            device.clone(),
            surface.clone(),
            caps.min_image_count,
            format,
            [initial_dimensions.0, initial_dimensions.1],
            1,
            usage,
            &queue,
            SurfaceTransform::Identity,
            alpha,
            PresentMode::Fifo,
            true,
            None,
        )
        .unwrap()
    };

    // We now create a buffer that will store the shape of our triangle.
    let vertex_buffer = {
        vulkano::impl_vertex!(Vertex, position);

        CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            [
                Vertex {
                    position: [-0.5, -0.25],
                },
                Vertex {
                    position: [0.0, 0.5],
                },
                Vertex {
                    position: [0.25, -0.1],
                },
            ]
            .iter()
            .cloned(),
        )
        .unwrap()
    };

    mod vs {
        vulkano_shaders::shader! {
            ty: "vertex",
            src: "
#version 450

layout(location = 0) in vec2 position;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
}"
        }
    }

    mod fs {
        vulkano_shaders::shader! {
            ty: "fragment",
            src: "
#version 450

layout(location = 0) out vec4 f_color;

void main() {
    f_color = vec4(1.0, 0.0, 0.0, 1.0);
}
"
        }
    }

    let vs = vs::Shader::load(device.clone()).unwrap();
    let fs = fs::Shader::load(device.clone()).unwrap();

    // The next step is to create a *render pass*, which is an object that describes where the
    // output of the graphics pipeline will go. It describes the layout of the images
    // where the colors, depth and/or stencil information will be written.
    let render_pass = Arc::new(
        vulkano::single_pass_renderpass!(
            device.clone(),
            attachments: {
                // `color` is a custom name we give to the first and only attachment.
                color: {
                    // `load: Clear` means that we ask the GPU to clear the content of this
                    // attachment at the start of the drawing.
                    load: Clear,
                    // `store: Store` means that we ask the GPU to store the output of the draw
                    // in the actual image. We could also ask it to discard the result.
                    store: Store,
                    // `format: <ty>` indicates the type of the format of the image. This has to
                    // be one of the types of the `vulkano::format` module (or alternatively one
                    // of your structs that implements the `FormatDesc` trait). Here we use the
                    // same format as the swapchain.
                    format: swapchain.format(),
                    samples: 1,
                }
            },
            pass: {
                // We use the attachment named `color` as the one and only color attachment.
                color: [color],
                // No depth-stencil attachment is indicated with empty brackets.
                depth_stencil: {}
            }
        )
        .unwrap(),
    );

    let pipeline = Arc::new(
        GraphicsPipeline::start()
            // We need to indicate the layout of the vertices.
            // The type `SingleBufferDefinition` actually contains a template parameter corresponding
            // to the type of each vertex. But in this code it is automatically inferred.
            .vertex_input_single_buffer::<Vertex>()
            .vertex_shader(vs.main_entry_point(), ())
            // The content of the vertex buffer describes a list of triangles.
            .triangle_list()
            // Use a resizable viewport set to draw over the entire window
            .viewports_dynamic_scissors_irrelevant(1)
            .fragment_shader(fs.main_entry_point(), ())
            // We have to indicate which subpass of which render pass this pipeline is going to be used
            // in. The pipeline will only be usable from this particular subpass.
            .render_pass(Subpass::from(render_pass.clone(), 0).unwrap())
            .build(device.clone())
            .unwrap(),
    );

    // Dynamic viewports allow us to recreate just the viewport when the window is resized
    // Otherwise we would have to recreate the whole pipeline.
    let mut dynamic_state = DynamicState {
        line_width: None,
        viewports: None,
        scissors: None,
    };

    // The render pass we created above only describes the layout of our framebuffers. Before we
    // can draw we also need to create the actual framebuffers.
    //
    // Since we need to draw to multiple images, we are going to create a different framebuffer for
    // each image.
    let framebuffers =
        window_size_dependent_setup(&images, render_pass.clone(), &mut dynamic_state);

    // In the loop below we are going to submit commands to the GPU. Submitting a command produces
    // an object that implements the `GpuFuture` trait, which holds the resources for as long as
    // they are in use by the GPU.
    //
    // Destroying the `GpuFuture` blocks until the GPU is finished executing it. In order to avoid
    // that, we store the submission of the previous frame here.
    let previous_frame_end = Box::new(sync::now(device.clone())) as Box<GpuFuture>;

    let context = RenderContext {
        surface,
        device,
        queue,
        vertex_buffer: vec![vertex_buffer],
        render_pass,
        pipeline,
    };

    let frame = FrameState {
        dynamic_state,
        recreate_swapchain: false,
        previous_frame_end: Some(previous_frame_end),
        swapchain,
        framebuffers,
    };

    (context, frame)
}

impl<W: Window> FrameState<W> {
    pub fn recreate_swapchain(&mut self) {
        self.recreate_swapchain = true
    }
}

impl<W: Window> RenderContext<W> {
    /// Render the frame's state.
    pub fn render(&self, frame: &mut FrameState<W>) {
        // It is important to call this function from time to time, otherwise resources will keep
        // accumulating and you will eventually reach an out of memory error.
        frame
            .previous_frame_end
            .as_mut()
            .unwrap()
            .cleanup_finished();

        let window = self.surface.window();

        // Whenever the window resizes we need to recreate everything dependent on the window size.
        // In this example that includes the swapchain, the framebuffers and the dynamic state viewport.
        while frame.recreate_swapchain {
            let dimensions = window.physical_size();

            let (new_swapchain, new_images) = match frame
                .swapchain
                .recreate_with_dimension([dimensions.0, dimensions.1])
            {
                Ok(r) => r,
                // This error tends to happen when the user is manually resizing the window.
                // Simply restarting the loop is the easiest way to fix this issue.
                Err(SwapchainCreationError::UnsupportedDimensions) => {
                    println!("unsupported dimensions {:?}, recreating", dimensions);
                    continue;
                }
                Err(err) => panic!("{:?}", err),
            };

            frame.swapchain = new_swapchain;
            frame.framebuffers = window_size_dependent_setup(
                &new_images,
                self.render_pass.clone(),
                &mut frame.dynamic_state,
            );

            frame.recreate_swapchain = false;
        }

        let (image_num, acquire_future) =
            swapchain::acquire_next_image(frame.swapchain.clone(), None).unwrap();

        // Specify the color to clear the framebuffer with i.e. blue
        let clear_values = vec![[1.0, 1.0, 1.0, 1.0].into()];

        let command_buffer = AutoCommandBufferBuilder::primary_one_time_submit(
            self.device.clone(),
            self.queue.family(),
        )
        .unwrap()
        .begin_render_pass(frame.framebuffers[image_num].clone(), false, clear_values)
        .unwrap()
        // We are now inside the first subpass of the render pass. We add a draw command.
        //
        // The last two parameters contain the list of resources to pass to the shaders.
        // Since we used an `EmptyPipeline` object, the objects have to be `()`.
        .draw(
            self.pipeline.clone(),
            &frame.dynamic_state,
            self.vertex_buffer.clone(),
            (),
            (),
        )
        .unwrap()
        // We leave the render pass by calling `draw_end`. Note that if we had multiple
        // subpasses we could have called `next_inline` (or `next_secondary`) to jump to the
        // next subpass.
        .end_render_pass()
        .unwrap()
        // Finish building the command buffer by calling `build`.
        .build()
        .unwrap();

        let future = frame
            .previous_frame_end
            .take()
            .unwrap()
            .join(acquire_future)
            .then_execute(self.queue.clone(), command_buffer)
            .unwrap()
            .then_swapchain_present(self.queue.clone(), frame.swapchain.clone(), image_num)
            .then_signal_fence_and_flush();

        frame.previous_frame_end = Some(match future {
            Ok(future) => Box::new(future) as Box<_>,
            Err(FlushError::OutOfDate) => {
                frame.recreate_swapchain = true;
                Box::new(sync::now(self.device.clone())) as Box<_>
            }
            Err(e) => {
                println!("{:?}", e);
                Box::new(sync::now(self.device.clone())) as Box<_>
            }
        })
    }
}

/// This method is called once during initialization, then again whenever the window is resized
fn window_size_dependent_setup<W: Window>(
    images: &[Arc<SwapchainImage<W>>],
    render_pass: Arc<RenderPassAbstract + Send + Sync>,
    dynamic_state: &mut DynamicState,
) -> Vec<Arc<FramebufferAbstract + Send + Sync>> {
    let dimensions = images[0].dimensions();

    let viewport = Viewport {
        origin: [0.0, 0.0],
        dimensions: [dimensions[0] as f32, dimensions[1] as f32],
        depth_range: 0.0..1.0,
    };
    dynamic_state.viewports = Some(vec![viewport]);

    images
        .iter()
        .map(|image| {
            Arc::new(
                Framebuffer::start(render_pass.clone())
                    .add(image.clone())
                    .unwrap()
                    .build()
                    .unwrap(),
            ) as Arc<FramebufferAbstract + Send + Sync>
        })
        .collect::<Vec<_>>()
}
