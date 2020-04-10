///! Vulkan <-> Skia <-> emergent::drawing interop.
use crate::renderer::{DrawingBackend, DrawingSurface, RenderContext};
use emergent::skia::convert::ToSkia;
use emergent::{text_as_lines, Frame, TextOrigin};
use emergent_drawing as drawing;
use emergent_drawing::text::With;
use emergent_drawing::{font, DrawTo, Shape, Transform};
use emergent_ui::{Window, DPI};
use skia_safe::gpu::vk;
use skia_safe::{
    gpu, Canvas, CanvasPointMode, Color, ColorType, Font, Paint, Shaper, Surface, Typeface, Vector,
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
                get_device_proc(device, name)
            }
        }
    }

    #[inline(never)]
    pub fn new_skia_backend(&self) -> Option<Backend> {
        let get_proc = |gpo| match self.get_proc(gpo) {
            Some(f) => f as _,
            None => {
                warn!("lookup failed for:");
                warn!("  {:?}", unsafe { gpo.name() });
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
        *queue.internal_object_guard() as _,
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
        let shaper = Shaper::new(None);
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

        // debug!(image_access.final_layout_requirement());
        // dbg!(image_access.initial_layout_requirement());

        let alloc = vk::Alloc::default();
        // TODO: verify TILING, IMAGE_LAYOUT and FORMAT assumptions.
        let image_info = &unsafe {
            vk::ImageInfo::new(
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

        // let view = View3D::default();
        // view.rotate_y(10.0);
        // view.rotate_x(10.0);

        // view.apply_to_canvas(canvas.borrow_mut());
        // canvas.scale((2.0, 2.0));

        let drawing_target =
            &mut CanvasDrawingTarget::from_canvas(canvas, frame.layout.dpi, &self.shaper);
        frame
            .presentation
            .draw_to(drawing::Paint::default(), drawing_target);

        surface.flush();
    }
}

impl DrawingSurface for skia_safe::Surface {}

struct CanvasDrawingTarget<'a> {
    canvas: &'a mut Canvas,
    // shaper: &'a shaper::Shaper,
    _dpi: DPI,
    paint: PaintSync,
    font: FontSync,
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
            Shape::Path(path) => {
                self.canvas
                    .draw_path(&path.to_skia(), self.paint.resolve(paint));
            }
            Shape::Image(_, _, _) => unimplemented!("image"),
            Shape::Text(drawing::Text { font, origin, runs }) => {
                let origin = TextOrigin::new(*origin);
                self.draw_text_runs(font, origin, runs, paint)
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

impl CanvasDrawingTarget<'_> {
    fn draw_text_runs(
        &mut self,
        font: &drawing::Font,
        mut origin: TextOrigin,
        runs: &[drawing::text::Run],
        paint: drawing::Paint,
    ) {
        for run in runs {
            origin = self.draw_text_run(font, origin, run, paint);
        }
    }

    fn draw_text_run(
        &mut self,
        font: &drawing::Font,
        origin: TextOrigin,
        run: &drawing::text::Run,
        paint: drawing::Paint,
    ) -> TextOrigin {
        use drawing::text::Run;
        let mut current = origin;

        match run {
            Run::Text(s, properties) => {
                // TODO: this clones the typeface string!
                let mut run_font = font.clone();
                if let Some(style) = properties.style {
                    run_font.style = style;
                }
                let font = self.font.resolve(&run_font);
                let line_spacing = font.spacing() as drawing::scalar;

                let mut last_line = None;
                for (i, line) in text_as_lines(&s).enumerate() {
                    if i != 0 {
                        current.newline(line_spacing);
                    }
                    let paint = self.paint.resolve(paint.with(*properties));
                    last_line = Some(line);
                    self.canvas
                        .draw_str(line, current.point().to_skia(), font, &paint);
                }

                if let Some(last_line) = last_line {
                    // TODO: use the measure result from the MeasureText implementation's cache here?
                    let last_line_advance = font.measure_str(last_line, None).0 as drawing::scalar;
                    current.advance(last_line_advance);
                };

                current
            }
            Run::Block(_) => unimplemented!(),
            Run::Drawing(_, _) => unimplemented!(),
        }
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

    fn from_canvas(canvas: &'a mut Canvas, dpi: DPI, _shaper: &'a Shaper) -> Self {
        let drawing_paint = drawing::Paint::default();

        Self {
            canvas,
            _dpi: dpi,
            // shaper,
            paint: PaintSync::from_paint(drawing_paint),
            // TODO: clarify if we need a notion of a default font.
            font: FontSync::new(dpi),
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
        paint.set_stroke_width(dp.width.to_skia());
        paint.set_stroke_miter(dp.miter.to_skia());
        paint.set_stroke_cap(dp.cap.to_skia());
        paint.set_stroke_join(dp.join.to_skia());
        paint.set_blend_mode(dp.blend_mode.to_skia());
    }
}

struct FontSync {
    dpi: DPI,
    drawing_font: drawing::Font,
    typeface: Typeface,
    font: Font,
}

impl FontSync {
    pub fn new(dpi: DPI) -> FontSync {
        // TODO: we need a notion of a default font.
        Self::from_font(
            dpi,
            &drawing::Font::new("", font::Style::NORMAL, font::Size::new(12.0)),
        )
    }

    pub fn from_font(dpi: DPI, font: &drawing::Font) -> FontSync {
        let (typeface, _sk_font) = Self::create_typeface_and_font(dpi, font);
        let sk_font = Font::from_typeface(&typeface, dpi.scale_font_points(*font.size) as f32);
        Self {
            dpi,
            drawing_font: font.clone(),
            typeface,
            font: sk_font,
        }
    }

    pub fn resolve(&mut self, font: &drawing::Font) -> &Font {
        if font.name != self.drawing_font.name || font.style != self.drawing_font.style {
            let (tf, f) = Self::create_typeface_and_font(self.dpi, font);
            self.typeface = tf;
            self.font = f;
            self.drawing_font = font.clone();
        } else if font.size != self.drawing_font.size {
            self.font = Font::from_typeface(
                &self.typeface,
                self.dpi.scale_font_points(*font.size) as f32,
            );
            self.drawing_font.size = font.size
        }

        &self.font
    }

    pub fn create_typeface_and_font(dpi: DPI, font: &drawing::Font) -> (Typeface, Font) {
        let typeface = Typeface::from_name(&font.name, font.style.to_skia()).unwrap_or_default();
        let sk_font = Font::from_typeface(&typeface, dpi.scale_font_points(*font.size) as f32);
        (typeface, sk_font)
    }
}
