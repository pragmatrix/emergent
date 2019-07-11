use crate::{BlendMode, Clip, Draw, Drawing, DrawingTarget, Paint, Shape, Transformation};

impl DrawingTarget for Drawing {
    fn fill(&mut self, paint: &Paint, blend_mode: BlendMode) {
        self.0.push(Draw::Paint(paint.clone(), blend_mode));
    }

    fn draw(&mut self, shape: &Shape, paint: &Paint) {
        match self.0.last_mut() {
            Some(Draw::Shapes(shapes, p)) if p == paint => {
                shapes.push(shape.clone());
            }
            _ => self
                .0
                .push(Draw::Shapes(vec![shape.clone()], paint.clone())),
        }
    }

    fn paint(&mut self, f: impl FnOnce(&mut Self)) {
        let begin = self.0.len();
        f(self);
        let nested = Drawing(self.0.drain(begin..).collect());
        if !nested.is_empty() {
            self.0.push(Draw::Drawing(nested));
        }
    }

    fn clip(&mut self, clip: &Clip, f: impl FnOnce(&mut Self)) {
        let begin = self.0.len();
        f(self);
        let nested = Drawing(self.0.drain(begin..).collect());
        if !nested.is_empty() {
            self.0.push(Draw::Clipped(clip.clone(), nested));
        }
    }

    fn transform(&mut self, transformation: &Transformation, f: impl FnOnce(&mut Self)) {
        let begin = self.0.len();
        f(self);
        let nested = Drawing(self.0.drain(begin..).collect());
        if !nested.is_empty() {
            self.0
                .push(Draw::Transformed(transformation.clone(), nested));
        }
    }
}
