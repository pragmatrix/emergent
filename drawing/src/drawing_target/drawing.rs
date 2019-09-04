use crate::{BlendMode, Clip, Drawing, DrawingTarget, Paint, ReplaceWith, Shape, Transform};

impl DrawingTarget for Drawing {
    fn fill(&mut self, paint: Paint, blend_mode: BlendMode) {
        self.replace_with(|s| s.below(Drawing::WithPaint(paint, Drawing::Fill(blend_mode).into())));
    }

    fn draw_shape(&mut self, shape: &Shape, paint: Paint) {
        self.replace_with(|s| {
            s.below(Drawing::WithPaint(
                paint,
                Drawing::Shape(shape.clone()).into(),
            ))
        });
    }

    fn clip(&mut self, clip: &Clip, f: impl FnOnce(&mut Self)) {
        let mut drawing = Drawing::new();
        f(&mut drawing);
        self.replace_with(|s| s.below(Drawing::Clipped(clip.clone(), drawing.into())));
    }

    fn transform(&mut self, transformation: &Transform, f: impl FnOnce(&mut Self)) {
        let mut drawing = Drawing::new();
        f(&mut drawing);
        self.replace_with(|s| {
            s.below(Drawing::Transformed(transformation.clone(), drawing.into()))
        });
    }
}
