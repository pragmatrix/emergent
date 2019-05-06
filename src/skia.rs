///! Vulkan <-> Skia interop.
use crate::renderer::{DrawingBackend, DrawingSurface, RenderContext, Window};
use skia_safe::gpu::vk;
use skia_safe::{gpu, scalar, Color, Paint};
use skia_safe::{ColorType, Surface};
use std::convert::TryInto;
use std::ffi::{c_void, CString};
use std::os::raw::c_char;
use std::sync::Arc;
use std::{mem, ptr};
use vulkano::framebuffer::FramebufferAbstract;
use vulkano::instance::loader;
use vulkano::{SynchronizedVulkanObject, VulkanObject};

type GetProcResult = Option<unsafe extern "system" fn() -> c_void>;
type GetDeviceProc = extern "system" fn(vk::Device, *const c_char) -> GetProcResult;

fn get_instance_proc(instance: vk::Instance, name: *const c_char) -> GetProcResult {
    let loader = loader::auto_loader().unwrap();
    unsafe { mem::transmute(loader.get_instance_proc_addr(instance as _, name)) }
}

impl<W: Window> RenderContext<W> {
    fn get_proc(&self, gpo: vk::GetProcOf) -> GetProcResult {
        match gpo {
            vk::GetProcOf::Instance(instance, name) => get_instance_proc(instance, name),
            vk::GetProcOf::Device(device, name) => {
                let instance = self.instance().internal_object() as _;
                // TODO: call this only once (or resolve it via vulkano, if that is possible).
                let get_device_proc: GetDeviceProc = unsafe {
                    let get_device_proc_addr_str = CString::new("vkGetDeviceProcAddr").unwrap();
                    mem::transmute(
                        get_instance_proc(instance, get_device_proc_addr_str.as_ptr()).unwrap(),
                    )
                };
                unsafe { mem::transmute(get_device_proc(device, name)) }
            }
        }
    }

    #[inline(never)]
    pub fn new_skia_context(&self) -> Option<gpu::Context> {
        let get_proc = |gpo| match self.get_proc(gpo) {
            Some(f) => f as _,
            None => {
                dbg!("lookup failed for");
                dbg!(unsafe { gpo.name() });
                ptr::null()
            }
        };

        let backend_context = new_backend_context(&get_proc, &self);
        gpu::Context::new_vulkan(&backend_context)
    }
}

fn new_backend_context<'lt, W: Window, GP: vk::GetProc>(
    get_proc: &'lt GP,
    render_context: &'lt RenderContext<W>,
) -> vk::BackendContext<'lt> {
    let instance: vk::Instance = render_context.instance().internal_object() as _;
    let physical_device = render_context.physical_device().internal_object() as _;
    let device = render_context.device.internal_object() as _;
    let queue = render_context.queue.clone();
    let (queue, queue_index) = (
        queue.internal_object_guard().clone() as _,
        queue.id_within_family() as _,
    );

    unsafe {
        vk::BackendContext::new(
            instance,
            physical_device,
            device,
            (queue, queue_index),
            get_proc,
        )
    }
}

//
// DrawingBakcend and other Traits to make Skia accessible to the renderer.
//

impl DrawingBackend for gpu::Context {
    type Surface = skia_safe::Surface;

    fn new_surface_from_framebuffer(
        &mut self,
        framebuffer: &Arc<FramebufferAbstract + Send + Sync>,
    ) -> Self::Surface {
        let [width, height, _] = framebuffer.dimensions();
        let image_access = framebuffer.attached_image_view(0).unwrap().parent();
        let image_object = image_access.inner().image.internal_object();

        let format = image_access.format();

        let (vk_format, color_type) = match format {
            vulkano::format::Format::B8G8R8A8Unorm => (
                skia_bindings::VkFormat::VK_FORMAT_B8G8R8A8_UNORM,
                ColorType::BGRA8888,
            ),
            _ => panic!("unsupported color format {:?}", format),
        };

        // dbg!(image_access.final_layout_requirement());
        // dbg!(image_access.initial_layout_requirement());

        let alloc = vk::Alloc::default();
        // TODO: verify TILING, IMAGE_LAYOUT and FORMAT assumptions.
        let image_info = &unsafe {
            vk::ImageInfo::from_image(
                image_object as _,
                alloc,
                skia_bindings::VkImageTiling::VK_IMAGE_TILING_OPTIMAL,
                skia_bindings::VkImageLayout::VK_IMAGE_LAYOUT_COLOR_ATTACHMENT_OPTIMAL,
                vk_format,
                1,
                None,
                None,
            )
        };

        let render_target = &gpu::BackendRenderTarget::new_vulkan(
            (width.try_into().unwrap(), height.try_into().unwrap()),
            0,
            image_info,
        );

        Surface::from_backend_render_target(
            self,
            render_target,
            gpu::SurfaceOrigin::TopLeft,
            color_type,
            None,
            None,
        )
        .unwrap()
    }
}

impl DrawingSurface for skia_safe::Surface {
    fn draw(&mut self) {
        let canvas = self.canvas();
        canvas.clear(Color::RED);
        let paint = &Paint::default();
        canvas.draw_circle((200, 200), 100.0, paint);
        self.flush();
    }
}
