use crate::emergent::Frame;
///! Vulkan <-> Skia interop.
use crate::renderer::{DrawingBackend, DrawingSurface, RenderContext, Window};
use emergent_drawing as drawing;
use emergent_drawing::{Circle, Radius, Shape};
use skia_safe::gpu::vk;
use skia_safe::{
    gpu, scalar, BlendMode, Canvas, Color, Paint, PaintCap, PaintJoin, PaintStyle, Point,
};
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
    fn draw(&mut self, frame: &Frame) {
        let canvas = self.canvas();
        canvas.clear(Color::WHITE);

        let drawing_target = &mut CanvasDrawingTarget::from_canvas(canvas);
        frame.draw(drawing_target);

        self.flush();
    }
}

struct CanvasDrawingTarget<'canvas> {
    canvas: &'canvas mut Canvas,
    paint: PaintSync,
}

impl<'a> drawing::DrawingTarget for CanvasDrawingTarget<'a> {
    fn fill(&mut self, paint: &drawing::Paint, blend_mode: drawing::BlendMode) {
        unimplemented!()
    }

    fn draw(&mut self, shape: drawing::Shape, paint: &drawing::Paint) {
        match shape {
            Shape::Point(_) => {}
            Shape::Points(_) => {}
            Shape::Line(_) => {}
            Shape::Lines(_) => {}
            Shape::LineSegments(_) => {}
            Shape::Polygon(_) => {}
            Shape::Rect(_) => {}
            Shape::Oval(_) => {}
            Shape::RoundedRect(_) => {}
            Shape::Circle(Circle(p, r)) => {
                self.canvas
                    .draw_circle(p.into_skia(), r.into_skia(), self.paint.resolve(paint));
            }
            Shape::Arc(_) => {}
            Shape::Path(_) => {}
            Shape::Image(_) => {}
            Shape::ImageRect(_, _, _) => {}
        }
    }

    fn clip(&mut self, clip: drawing::Clip) {
        unimplemented!()
    }

    fn transform(&mut self, transformation: drawing::Transformation) {
        unimplemented!()
    }
}

impl<'a> CanvasDrawingTarget<'a> {
    fn canvas(&mut self) -> &mut Canvas {
        &mut self.canvas
    }

    fn from_canvas(canvas: &'a mut Canvas) -> Self {
        let drawing_paint = drawing::Paint::default();

        Self {
            canvas,
            paint: PaintSync::from_paint(drawing_paint),
        }
    }
}

struct PaintSync {
    drawing_paint: drawing::Paint,
    paint: Paint,
}

impl PaintSync {
    fn from_paint(drawing_paint: drawing::Paint) -> PaintSync {
        let mut paint = Paint::default();
        paint.set_anti_alias(true);
        PaintSync::apply_paint(&mut paint, &drawing_paint);
        PaintSync {
            paint,
            drawing_paint,
        }
    }

    fn resolve(&mut self, paint: &drawing::Paint) -> &Paint {
        if *paint != self.drawing_paint {
            Self::apply_paint(&mut self.paint, paint);
            self.drawing_paint = paint.clone();
        }
        &self.paint
    }

    // defaults are here: https://skia.org/user/api/SkPaint_Reference
    // TODO: resolve the individual defaults and store them locally.
    fn apply_paint(paint: &mut Paint, dp: &drawing::Paint) {
        // TODO: we _do_ know which values have been changed, so probably we should apply only that.
        paint.set_style(dp.style.unwrap_or(drawing::PaintStyle::Fill).into_skia());
        // TODO: specify the default on the drawing:: side.
        paint.set_color(dp.color.map(|c| c.into_skia()).unwrap_or(Color::BLACK));
        paint.set_stroke_width(dp.stroke_width.unwrap_or(0.0));
        paint.set_stroke_miter(dp.stroke_miter.unwrap_or(4.0));
        paint.set_stroke_cap(
            dp.stroke_cap
                .unwrap_or(drawing::StrokeCap::Butt)
                .into_skia(),
        );
        paint.set_stroke_join(
            dp.stroke_join
                .unwrap_or(drawing::StrokeJoin::Miter)
                .into_skia(),
        );
        paint.set_blend_mode(
            dp.blend_mode
                .unwrap_or(drawing::BlendMode::SourceOver)
                .into_skia(),
        );
    }
}

//
// IntoSkia implementations
//

trait IntoSkia<ST> {
    fn into_skia(self) -> ST;
}

impl IntoSkia<Point> for drawing::Point {
    fn into_skia(self) -> Point {
        let drawing::Point(x, y) = self;
        Point::from((x, y))
    }
}

impl IntoSkia<Color> for drawing::Color {
    fn into_skia(self) -> Color {
        Color::from(self.0)
    }
}

impl IntoSkia<f32> for drawing::Radius {
    fn into_skia(self) -> scalar {
        self.0
    }
}

impl IntoSkia<BlendMode> for drawing::BlendMode {
    fn into_skia(self) -> BlendMode {
        BLEND_MODE_TABLE[self as usize]
    }
}

// TODO: can we statically verfiy the of this table?
const BLEND_MODE_TABLE: [skia_safe::BlendMode; 18] = [
    BlendMode::Src,
    BlendMode::SrcOver,
    BlendMode::SrcIn,
    BlendMode::SrcATop,
    BlendMode::Dst,
    BlendMode::DstOver,
    BlendMode::DstIn,
    BlendMode::DstATop,
    BlendMode::Clear,
    BlendMode::SrcOut,
    BlendMode::DstOut,
    BlendMode::Xor,
    BlendMode::Darken,
    BlendMode::Lighten,
    BlendMode::Multiply,
    BlendMode::Screen,
    BlendMode::Overlay,
    BlendMode::Plus,
];

impl IntoSkia<PaintStyle> for drawing::PaintStyle {
    fn into_skia(self) -> PaintStyle {
        match self {
            drawing::PaintStyle::Stroke => PaintStyle::Stroke,
            drawing::PaintStyle::Fill => PaintStyle::Fill,
            drawing::PaintStyle::StrokeAndFill => PaintStyle::StrokeAndFill,
        }
    }
}

impl IntoSkia<PaintCap> for drawing::StrokeCap {
    fn into_skia(self) -> PaintCap {
        match self {
            drawing::StrokeCap::Butt => PaintCap::Butt,
            drawing::StrokeCap::Round => PaintCap::Round,
            drawing::StrokeCap::Square => PaintCap::Square,
        }
    }
}

impl IntoSkia<PaintJoin> for drawing::StrokeJoin {
    fn into_skia(self) -> PaintJoin {
        match self {
            drawing::StrokeJoin::Miter => PaintJoin::Miter,
            drawing::StrokeJoin::Round => PaintJoin::Round,
            drawing::StrokeJoin::Bevel => PaintJoin::Bevel,
        }
    }
}
