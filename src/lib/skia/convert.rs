use emergent_drawing as drawing;
use skia_safe::{
    font_style, scalar, BlendMode, Color, FontStyle, PaintCap, PaintJoin, PaintStyle, Path, Point,
    RRect, Rect, Size, Vector,
};

pub trait ToSkia<ST> {
    fn to_skia(&self) -> ST;
}

impl ToSkia<Color> for drawing::Color {
    fn to_skia(&self) -> Color {
        Color::from(self.to_u32())
    }
}

impl ToSkia<Point> for drawing::Point {
    fn to_skia(&self) -> Point {
        Point::from((self.x.to_skia(), self.y.to_skia()))
    }
}

impl ToSkia<Vector> for drawing::Vector {
    fn to_skia(&self) -> Point {
        Vector::from((self.x.to_skia(), self.y.to_skia()))
    }
}

impl ToSkia<Vec<Point>> for [drawing::Point] {
    fn to_skia(&self) -> Vec<Point> {
        self.iter().map(|p| (*p).to_skia()).collect()
    }
}

impl ToSkia<Size> for drawing::Vector {
    fn to_skia(&self) -> Size {
        Size::from((self.x.to_skia(), self.y.to_skia()))
    }
}

impl ToSkia<Vector> for drawing::Extent {
    fn to_skia(&self) -> Vector {
        Vector::new(self.width.to_skia(), self.height.to_skia())
    }
}

impl ToSkia<Rect> for drawing::Rect {
    fn to_skia(&self) -> Rect {
        Rect::from((self.left_top().to_skia(), self.size().to_skia()))
    }
}

impl ToSkia<RRect> for drawing::RoundedRect {
    fn to_skia(&self) -> RRect {
        let corners = self.corner_radii();
        let corners = [
            corners[0].to_skia(),
            corners[1].to_skia(),
            corners[2].to_skia(),
            corners[3].to_skia(),
        ];
        RRect::new_rect_radii(self.rect().to_skia(), &corners)
    }
}

impl ToSkia<Path> for drawing::Path {
    fn to_skia(&self) -> Path {
        let mut path = Path::new();
        for verb in self.verbs() {
            use drawing::path::Verb::*;
            match verb {
                MoveTo(p) => path.move_to(p.to_skia()),
                LineTo(p2) => path.line_to(p2.to_skia()),
                QuadTo(p2, p3) => path.quad_to(p2.to_skia(), p3.to_skia()),
                ConicTo(p2, p3, w) => path.conic_to(p2.to_skia(), p3.to_skia(), *w as scalar),
                CubicTo(p2, p3, p4) => path.cubic_to(p2.to_skia(), p3.to_skia(), p4.to_skia()),
                Close => path.close(),
            };
        }
        path
    }
}

impl ToSkia<f32> for drawing::Radius {
    fn to_skia(&self) -> scalar {
        (**self).to_skia()
    }
}

impl ToSkia<BlendMode> for drawing::BlendMode {
    fn to_skia(&self) -> BlendMode {
        BLEND_MODE_TABLE[*self as usize]
    }
}

const BLEND_MODE_TABLE: [BlendMode; 18] = [
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

impl ToSkia<PaintStyle> for drawing::paint::Style {
    fn to_skia(&self) -> PaintStyle {
        match self {
            drawing::paint::Style::Stroke => PaintStyle::Stroke,
            drawing::paint::Style::Fill => PaintStyle::Fill,
            drawing::paint::Style::FillAndStroke => PaintStyle::StrokeAndFill,
        }
    }
}

impl ToSkia<PaintCap> for drawing::paint::Cap {
    fn to_skia(&self) -> PaintCap {
        use drawing::paint::Cap::*;
        match self {
            Butt => PaintCap::Butt,
            Round => PaintCap::Round,
            Square => PaintCap::Square,
        }
    }
}

impl ToSkia<PaintJoin> for drawing::paint::Join {
    fn to_skia(&self) -> PaintJoin {
        use drawing::paint::Join::*;
        match self {
            Miter => PaintJoin::Miter,
            Round => PaintJoin::Round,
            Bevel => PaintJoin::Bevel,
        }
    }
}

impl ToSkia<FontStyle> for drawing::font::Style {
    fn to_skia(&self) -> FontStyle {
        FontStyle::new(
            self.weight.to_skia(),
            self.width.to_skia(),
            self.slant.to_skia(),
        )
    }
}

impl ToSkia<font_style::Weight> for drawing::font::Weight {
    fn to_skia(&self) -> font_style::Weight {
        font_style::Weight::from(**self as i32)
    }
}

impl ToSkia<font_style::Width> for drawing::font::Width {
    fn to_skia(&self) -> font_style::Width {
        font_style::Width::from(**self as i32)
    }
}

impl ToSkia<font_style::Slant> for drawing::font::Slant {
    fn to_skia(&self) -> font_style::Slant {
        match self {
            drawing::font::Slant::Upright => font_style::Slant::Upright,
            drawing::font::Slant::Italic => font_style::Slant::Italic,
            drawing::font::Slant::Oblique => font_style::Slant::Oblique,
        }
    }
}

impl ToSkia<scalar> for drawing::scalar {
    fn to_skia(&self) -> f32 {
        // TODO: perf, also truncation?
        *self as f32
    }
}
