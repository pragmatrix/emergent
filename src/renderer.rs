use emergent::Frame;
use emergent_ui::Window;
use std::sync::Arc;
use vulkano::buffer::{BufferAccess, BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::DynamicState;
use vulkano::device::{Device, DeviceExtensions, Queue};
use vulkano::framebuffer::{Framebuffer, FramebufferAbstract, RenderPassAbstract, Subpass};
use vulkano::image::SwapchainImage;
use vulkano::instance::{ApplicationInfo, Instance, PhysicalDevice, Version};
use vulkano::pipeline::viewport::Viewport;
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::swapchain;
use vulkano::swapchain::ColorSpace;
use vulkano::swapchain::{
    PresentMode, Surface, SurfaceTransform, Swapchain, SwapchainCreationError,
};
use vulkano::sync;
use vulkano::sync::{FlushError, GpuFuture};

#[derive(Debug, Clone, Default)]
struct Vertex {
    position: [f32; 2],
}

pub struct RenderContext<W: Window> {
    physical_device_index: usize,
    surface: Arc<Surface<W>>,
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
    _vertex_buffer: Vec<Arc<dyn BufferAccess + Send + Sync>>,
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    _pipeline: Arc<dyn GraphicsPipelineAbstract + Send + Sync>,
}

pub struct FrameState<W: Window> {
    dynamic_state: DynamicState,
    swapchain: Arc<Swapchain<W>>,
    framebuffers: Vec<Arc<dyn FramebufferAbstract + Send + Sync>>,
}

pub trait DrawingSurface {}

pub trait DrawingBackend {
    type Surface: DrawingSurface;

    /// Creates a DrawingSurface that draws to a framebuffer.
    fn new_surface_from_framebuffer(
        &mut self,
        framebuffer: &Arc<dyn FramebufferAbstract + Send + Sync>,
    ) -> Self::Surface;

    /// Draws a frame on the Surface.
    fn draw(&self, frame: &Frame, surface: &mut Self::Surface);
}

pub fn new_instance() -> Arc<Instance> {
    let extensions = vulkano_win::required_extensions();
    let application_info = &ApplicationInfo {
        api_version: Version {
            major: 1,
            minor: 1,
            patch: 0,
        }
        .into(),
        ..ApplicationInfo::default()
    };

    Instance::new(Some(application_info), &extensions, None).unwrap()
}

pub fn create_context_and_frame_state<W: Window>(
    instance: Arc<Instance>,
    surface: Arc<Surface<W>>,
) -> (RenderContext<W>, FrameState<W>) {
    // TODO: select a proper physical device, the first one might not be suitable for
    //       rendering on the screen.
    let physical_device = PhysicalDevice::enumerate(&instance).next().unwrap();
    info!(
        "Using device: {} (type: {:?})",
        physical_device.name(),
        physical_device.ty()
    );

    let queue_family = physical_device
        .queue_families()
        .find(|&q| q.supports_graphics() && surface.is_supported(q).unwrap_or(false))
        .unwrap();

    let device_ext = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::none()
    };
    let (device, mut queues) = Device::new(
        physical_device,
        physical_device.supported_features(),
        &device_ext,
        [(queue_family, 0.5)].iter().cloned(),
    )
    .unwrap();

    let queue = queues.next().unwrap();

    let (swapchain, images) = {
        let caps = surface.capabilities(physical_device).unwrap();

        let usage = caps.supported_usage_flags;

        let alpha = caps.supported_composite_alpha.iter().next().unwrap();
        let format = caps.supported_formats[0].0;

        let window = surface.window();
        let initial_dimensions = window.frame_layout().dimensions;

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
            ColorSpace::SrgbNonLinear,
        )
        .unwrap()
    };

    // We now create a buffer that will store the shape of our triangle.
    let vertex_buffer = {
        vulkano::impl_vertex!(Vertex, position);

        CpuAccessibleBuffer::from_iter(
            device.clone(),
            BufferUsage::all(),
            true, // host_cached?
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
                    load: DontCare,
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
        compare_mask: None,
        write_mask: None,
        reference: None,
    };

    // The render pass we created above only describes the layout of our framebuffers. Before we
    // can draw we also need to create the actual framebuffers.
    //
    // Since we need to draw to multiple images, we are going to create a different framebuffer for
    // each image.
    let framebuffers =
        window_size_dependent_setup(&images, render_pass.clone(), &mut dynamic_state);

    let context = RenderContext {
        surface,
        physical_device_index: physical_device.index(),
        device,
        queue,
        _vertex_buffer: vec![vertex_buffer],
        render_pass,
        _pipeline: pipeline,
    };

    let frame = FrameState {
        dynamic_state,
        swapchain,
        framebuffers,
    };

    (context, frame)
}

impl<W: Window> RenderContext<W> {
    /// Returns the physical device.
    pub fn physical_device(&self) -> PhysicalDevice {
        PhysicalDevice::from_index(self.instance(), self.physical_device_index).unwrap()
    }

    pub fn instance(&self) -> &Arc<Instance> {
        self.device.instance()
    }

    /// Renders a frame, updates the frame state and returns a future that gets
    /// fulfilled when the frame is on screen.
    pub fn render(
        &self,
        mut previous_render: Box<dyn GpuFuture>,
        frame_state: &mut FrameState<W>,
        drawing_backend: &mut impl DrawingBackend,
        frame: &Frame,
    ) -> Box<dyn GpuFuture> {
        previous_render.cleanup_finished();

        loop {
            match self.draw_and_present(previous_render, frame_state, drawing_backend, frame) {
                Ok(future) => return future,
                Err(FlushError::OutOfDate) => {
                    self.recreate_swapchain(frame_state);
                    previous_render = Box::new(sync::now(self.device.clone()));
                    continue;
                }
                Err(e) => {
                    error!("{:?}", e);
                    return Box::new(sync::now(self.device.clone()));
                }
            }
        }
    }

    /// Draw the frame's state.
    pub fn draw_and_present(
        &self,
        previous: Box<dyn GpuFuture>,
        frame_state: &mut FrameState<W>,
        drawing_backend: &mut impl DrawingBackend,
        frame: &Frame,
    ) -> Result<Box<dyn GpuFuture>, FlushError> {
        // for some reason we can't join this with acquire_future and drop it afterwards.
        drop(previous);

        let image_num = self.acquire_next_fb(frame_state);

        let framebuffer = &frame_state.framebuffers[image_num];

        {
            let mut surface = drawing_backend.new_surface_from_framebuffer(framebuffer);
            drawing_backend.draw(frame, &mut surface);
        }

        self.present(frame_state, image_num)
    }

    pub fn acquire_next_fb(&self, frame: &mut FrameState<W>) -> usize {
        let (image_num, suboptimal, acquire_future) =
            swapchain::acquire_next_image(frame.swapchain.clone(), None).unwrap();

        if suboptimal {
            debug!("acquired a suboptimal swapchain image");
        }

        // drop(previous.join(acquire_future));
        drop(acquire_future);
        image_num
    }

    pub fn present(
        &self,
        frame: &mut FrameState<W>,
        image_num: usize,
    ) -> Result<Box<dyn GpuFuture>, FlushError> {
        let future: Box<dyn GpuFuture> =
            Box::new(sync::now(self.device.clone()).then_swapchain_present(
                self.queue.clone(),
                frame.swapchain.clone(),
                image_num,
            ));

        Ok(future)
    }

    /// Returns true if the dimensions of the swapchain do not match the window's physical size.
    pub fn need_to_recreate_swapchain(&self, frame: &FrameState<W>) -> bool {
        let window = self.surface.window();
        let win_size = window.frame_layout().dimensions;
        let sc_size = {
            let [width, height] = frame.swapchain.dimensions();
            (width, height)
        };

        win_size != sc_size
    }

    pub fn recreate_swapchain(&self, frame: &mut FrameState<W>) {
        debug!("recreating swapchain");
        let window = self.surface.window();
        let dimensions = window.frame_layout().dimensions;

        let (new_swapchain, new_images) = match frame
            .swapchain
            .recreate_with_dimension([dimensions.0, dimensions.1])
        {
            Ok(r) => r,
            // This error tends to happen when the user is manually resizing the window.
            // Simply restarting the loop is the easiest way to fix this issue.
            Err(SwapchainCreationError::UnsupportedDimensions) => {
                warn!(
                    "unsupported dimensions {:?}, recreating swapchain",
                    dimensions
                );
                // TODO: don't recurse here, use a loop.
                return self.recreate_swapchain(frame);
            }
            Err(err) => panic!("{:?}", err),
        };

        frame.swapchain = new_swapchain;
        frame.framebuffers = window_size_dependent_setup(
            &new_images,
            self.render_pass.clone(),
            &mut frame.dynamic_state,
        );
    }
}

/// This method is called once during initialization, then again whenever the window is resized
fn window_size_dependent_setup<W: Window>(
    images: &[Arc<SwapchainImage<W>>],
    render_pass: Arc<dyn RenderPassAbstract + Send + Sync>,
    dynamic_state: &mut DynamicState,
) -> Vec<Arc<dyn FramebufferAbstract + Send + Sync>> {
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
            ) as Arc<dyn FramebufferAbstract + Send + Sync>
        })
        .collect::<Vec<_>>()
}
