///! Vulkan <-> Skia <-> emergent::drawing interop.
use crate::frame::Frame;
use crate::renderer::{DrawingBackend, DrawingSurface, RenderContext, Window};
use core::borrow::BorrowMut;
use emergent::skia::convert::ToSkia;
use emergent::text_as_lines;
use emergent_drawing as drawing;
use emergent_drawing::text::With;
use emergent_drawing::{DrawTo, Shape, Transform};
use skia_safe::gpu::vk;
use skia_safe::utils::View3D;
use skia_safe::{
    gpu, shaper, Canvas, CanvasPointMode, Color, ColorType, Font, Paint, Point, Shaper, Surface,
    Typeface, Vector,
};
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
    pub fn new_skia_backend(&self) -> Option<Backend> {
        let get_proc = |gpo| match self.get_proc(gpo) {
            Some(f) => f as _,
            None => {
                dbg!("lookup failed for");
                dbg!(unsafe { gpo.name() });
                ptr::null()
            }
        };

        let backend_context = new_backend_context(&get_proc, &self);
        gpu::Context::new_vulkan(&backend_context).map(Backend::from_context)
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

/// The Skia backend.
pub struct Backend {
    context: gpu::Context,
    shaper: Shaper,
}

impl Backend {
    pub fn from_context(context: gpu::Context) -> Self {
        let shaper = Shaper::new();
        Backend { context, shaper }
    }
}

//
// DrawingBackend and other Traits to make Skia accessible to the renderer.
//

impl DrawingBackend for Backend {
    type Surface = skia_safe::Surface;

    fn new_surface_from_framebuffer(
        &mut self,
        framebuffer: &Arc<dyn FramebufferAbstract + Send + Sync>,
    ) -> Self::Surface {
        let [width, height, _] = framebuffer.dimensions();
        let image_access = framebuffer.attached_image_view(0).unwrap().parent();
        let image_object = image_access.inner().image.internal_object();

        let format = image_access.format();

        let (vk_format, color_type) = match format {
            vulkano::format::Format::B8G8R8A8Unorm => {
                (vk::Format::B8G8R8A8_UNORM, ColorType::BGRA8888)
            }
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
                vk::ImageTiling::OPTIMAL,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                vk_format,
                1,
                None,
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
            &mut self.context,
            render_target,
            gpu::SurfaceOrigin::TopLeft,
            color_type,
            None,
            None,
        )
        .unwrap()
    }

    fn draw(&self, frame: &Frame, surface: &mut Surface) {
        let canvas = surface.canvas();
        canvas.clear(Color::WHITE);

        // let matrix44la = look_at((0.3, 0.5, 1.0), (0.0, 0.0, 0.0), (0.0, 1.0, 0.0));
        // let matrix44 = perspective(1.0, 4.0, std::f32::consts::PI / 5.0);
        // dbg!(matrix44.has_perspective());
        // let result = matrix44la * matrix44;
        // dbg!(result.has_perspective());

        let view = View3D::default();
        // view.rotate_y(10.0);
        // view.rotate_x(10.0);

        view.apply_to_canvas(canvas.borrow_mut());
        canvas.scale((2.0, 2.0));

        let drawing_target = &mut CanvasDrawingTarget::from_canvas(canvas, &self.shaper);
        frame.drawing.draw_to(drawing_target);

        surface.flush();
    }
}

impl DrawingSurface for skia_safe::Surface {}

struct CanvasDrawingTarget<'a> {
    canvas: &'a mut Canvas,
    shaper: &'a shaper::Shaper,
    paint: PaintSync,
    font: Option<FontSync>,
}

impl<'a> drawing::DrawingTarget for CanvasDrawingTarget<'a> {
    fn fill(&mut self, _paint: drawing::Paint, _blend_mode: drawing::BlendMode) {
        unimplemented!("fill")
    }

    fn draw_shape(&mut self, shape: &drawing::Shape, paint: drawing::Paint) {
        match shape {
            Shape::Point(p) => {
                self.canvas
                    .draw_point(p.to_skia(), self.paint.resolve(paint));
            }
            Shape::Line(drawing::Line { point1, point2 }) => {
                self.canvas.draw_line(
                    point1.to_skia(),
                    point2.to_skia(),
                    self.paint.resolve(paint),
                );
            }
            Shape::Polygon(polygon) => {
                self.canvas.draw_points(
                    CanvasPointMode::Polygon,
                    polygon.points().to_skia().as_slice(),
                    self.paint.resolve(paint),
                );
            }
            Shape::Rect(rect) => {
                self.canvas
                    .draw_rect(rect.to_skia(), self.paint.resolve(paint));
            }
            Shape::Oval(oval) => {
                self.canvas
                    .draw_oval(oval.rect().to_skia(), self.paint.resolve(paint));
            }
            Shape::RoundedRect(rounded_rect) => {
                self.canvas
                    .draw_rrect(rounded_rect.to_skia(), self.paint.resolve(paint));
            }
            Shape::Circle(c) => {
                self.canvas.draw_circle(
                    c.center.to_skia(),
                    c.radius.to_skia(),
                    self.paint.resolve(paint),
                );
            }
            Shape::Arc(_) => unimplemented!("arc"),
            Shape::Path(_) => unimplemented!("path"),
            Shape::Image(_, _, _) => unimplemented!("image"),
            Shape::Text(drawing::Text { font, origin, runs }) => {
                let font = FontSync::resolve_opt(&mut self.font, font);

                let origin = TextOrigin::new(*origin);

                draw_text_runs(
                    &self.shaper,
                    font,
                    origin,
                    runs,
                    paint,
                    &mut self.paint,
                    &mut self.canvas,
                )
            }
        }
    }

    fn clip(&mut self, _clip: &drawing::Clip, _f: impl FnOnce(&mut Self)) {
        unimplemented!("clip")
    }

    fn transform(
        &mut self,
        transformation: &drawing::Transform,
        draw_nested: impl FnOnce(&mut Self),
    ) {
        match transformation {
            Transform::Identity => draw_nested(self),
            Transform::Translate(v) => {
                self.canvas.save();
                let d: Vector = v.to_skia();
                self.canvas.translate(d);
                draw_nested(self);
                self.canvas.restore();
            }
            Transform::Scale(_, _) => unimplemented!("scale"),
            Transform::Rotate(_, _) => unimplemented!("rotate"),
            Transform::Matrix(_) => unimplemented!("matrix"),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
struct TextOrigin {
    /// Origin of the textbox.
    origin: drawing::Point,
    /// The beginning of the current line and advance relative to the origin.
    advance: drawing::Vector,
}

impl TextOrigin {
    pub fn new(origin: drawing::Point) -> TextOrigin {
        let advance = Default::default();
        TextOrigin { origin, advance }
    }

    pub fn point(&self) -> drawing::Point {
        self.origin + self.advance
    }
}

fn draw_text_runs(
    shaper: &Shaper,
    font: &Font,
    mut origin: TextOrigin,
    runs: &[drawing::text::Run],
    paint: drawing::Paint,
    paint_sync: &mut PaintSync,
    canvas: &mut Canvas,
) {
    for run in runs {
        origin = draw_text_run(shaper, &font, origin, run, paint, paint_sync, canvas);
    }
}

fn draw_text_run(
    shaper: &Shaper,
    font: &Font,
    origin: TextOrigin,
    run: &drawing::text::Run,
    paint: drawing::Paint,
    paint_sync: &mut PaintSync,
    canvas: &mut Canvas,
) -> TextOrigin {
    use drawing::text::Run;

    let line_spacing = font.spacing() as drawing::scalar;

    let mut current = origin;

    match run {
        Run::Text(s, properties) => {
            let mut last_line_advance = 0.0;
            for (i, line) in text_as_lines(&s).enumerate() {
                if i != 0 {
                    // add a newline: update the current line advance and reset x advance to zero.
                    current.advance = drawing::Vector::new(
                        0.0,
                        origin.advance.y + line_spacing * i as drawing::scalar,
                    )
                }
                let paint = paint_sync.resolve(paint.with(*properties));
                // TODO: we only need that for the last line.
                last_line_advance = font.measure_str(line, None).0 as drawing::scalar;
                canvas.draw_str(line, current.point().to_skia(), font, &paint);
            }
            current.advance += drawing::Vector::new(last_line_advance, 0.0);
            current
        }
        Run::Block(_) => unimplemented!(),
        Run::Drawing(_, _) => unimplemented!(),
    }
}

/*
/// Shape and draw a text run.
fn draw_text_run(
    shaper: &Shaper,
    text: &str,
    font: &Font,
    origin: Point,
    run: &drawing::text::Run,
    paint: drawing::Paint,
    paint_sync: &mut PaintSync,
    canvas: &mut Canvas,
) -> Point {
    match run {
        drawing::text::Run::Text(range, properties) => {
            let paint = paint_sync.resolve(paint.with(*properties));

            let (text_blob, end_point) =
                // TODO: support max width, right to left / bidi text..
                shaper.shape_text_blob(&text[range.clone()], font, true, std::f32::INFINITY, origin).unwrap();
            canvas.draw_text_blob(&text_blob, Point::default(), paint);
            end_point
        }
        drawing::text::Run::EndOfLine => {
            dbg!("unimplemented: EndOfLine");
            origin
        }
        drawing::text::Run::Block(_) => unimplemented!("text::Run::Block"),
        drawing::text::Run::Drawing(_, _) => unimplemented!("text::Run::Drawing"),
    }
}

*/

impl<'a> CanvasDrawingTarget<'a> {
    fn _canvas(&mut self) -> &mut Canvas {
        &mut self.canvas
    }

    fn from_canvas(canvas: &'a mut Canvas, shaper: &'a Shaper) -> Self {
        let drawing_paint = drawing::Paint::default();

        Self {
            canvas,
            shaper,
            paint: PaintSync::from_paint(drawing_paint),
            font: None,
        }
    }

    fn _resolve_font(&mut self, font: &drawing::Font) -> &Font {
        if self.font.is_none() {
            self.font = Some(FontSync::from_font(font));
        };

        self.font.as_mut().unwrap().resolve(font)
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
        PaintSync::apply_paint(&mut paint, drawing_paint);
        PaintSync {
            paint,
            drawing_paint,
        }
    }

    fn resolve(&mut self, paint: drawing::Paint) -> &Paint {
        if paint != self.drawing_paint {
            Self::apply_paint(&mut self.paint, paint);
            self.drawing_paint = paint;
        }
        &self.paint
    }

    // defaults are here: https://skia.org/user/api/SkPaint_Reference
    // TODO: resolve the individual defaults and store them locally.
    fn apply_paint(paint: &mut Paint, dp: drawing::Paint) {
        // TODO: we _do_ know which values have been changed, so probably we should apply only that.
        paint.set_style(dp.style.to_skia());
        paint.set_color(dp.color.to_skia());
        paint.set_stroke_width(dp.stroke_width.to_skia());
        paint.set_stroke_miter(dp.stroke_miter.to_skia());
        paint.set_stroke_cap(dp.stroke_cap.to_skia());
        paint.set_stroke_join(dp.stroke_join.to_skia());
        paint.set_blend_mode(dp.blend_mode.to_skia());
    }
}

struct FontSync {
    drawing_font: drawing::Font,
    typeface: Typeface,
    font: Font,
}

impl FontSync {
    pub fn from_font(font: &drawing::Font) -> FontSync {
        let (typeface, _sk_font) = Self::create_typeface_and_font(font);
        let sk_font = Font::from_typeface(&typeface, font.size.to_skia());
        Self {
            drawing_font: font.clone(),
            typeface,
            font: sk_font,
        }
    }

    pub fn resolve_opt<'a>(fso: &'a mut Option<FontSync>, font: &drawing::Font) -> &'a Font {
        match fso {
            None => {
                *fso = Some(Self::from_font(font));
                fso.as_mut().unwrap().resolve(font)
            }

            Some(fs) => fs.resolve(font),
        }
    }

    pub fn resolve(&mut self, font: &drawing::Font) -> &Font {
        if font.name != self.drawing_font.name || font.style != self.drawing_font.style {
            let (tf, f) = Self::create_typeface_and_font(font);
            self.typeface = tf;
            self.font = f;
            self.drawing_font = font.clone();
        } else if font.size != self.drawing_font.size {
            self.font = Font::from_typeface(&self.typeface, font.size.to_skia());
            self.drawing_font.size = font.size
        }

        &self.font
    }

    pub fn create_typeface_and_font(font: &drawing::Font) -> (Typeface, Font) {
        let typeface = Typeface::from_name(&font.name, font.style.to_skia()).unwrap_or_default();
        let sk_font = Font::from_typeface(&typeface, font.size.to_skia());
        (typeface, sk_font)
    }
}
