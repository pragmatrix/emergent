use crate::{BlendMode, Clip, Draw, Drawing, DrawingTarget, Paint, Shape, Transform};

impl DrawingTarget for Drawing {
    fn fill(&mut self, paint: Paint, blend_mode: BlendMode) {
        self.push(Draw::Paint(paint, blend_mode));
    }

    fn draw_shape(&mut self, shape: &Shape, paint: Paint) {
        match self.last_mut() {
            Some(Draw::Shapes(shapes, p)) if *p == paint => {
                shapes.push(shape.clone());
            }
            _ => self.push(Draw::Shapes(vec![shape.clone()], paint)),
        }
    }

    fn clip(&mut self, clip: &Clip, f: impl FnOnce(&mut Self)) {
        let begin = self.len();
        f(self);
        let nested = Drawing::from(self.drain(begin..));
        if !nested.is_empty() {
            self.push(Draw::Clipped(clip.clone(), nested));
        }
    }

    fn transform(&mut self, transformation: &Transform, f: impl FnOnce(&mut Self)) {
        let begin = self.len();
        f(self);
        let nested = Drawing::from(self.drain(begin..));
        if !nested.is_empty() {
            self.push(Draw::Transformed(transformation.clone(), nested));
        }
    }
}
