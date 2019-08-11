#[cfg(test)]
mod tests {
    use crate::skia;
    use emergent_drawing::functions::*;
    use emergent_drawing::{
        font, paint, Color, Drawing, DrawingFastBounds, DrawingTarget, Font, Paint, Point, Rect,
        Render, Vector,
    };

    #[test]
    fn draw_circle() {
        let mut drawing = Drawing::new();
        drawing.draw(circle((100, 100), 100), paint().color(0xff0000ff));
        drawing.render();
    }

    #[test]
    fn text_bounds() {
        let measure = skia::measure::Measure::new();
        let mut drawing = Drawing::new();
        let font = Font::new("", font::Style::default(), font::Size::new(20.0));
        let text = text("Hello World", &font, None);
        drawing.draw(text, paint());
        let bounds: Rect = (*drawing.fast_bounds(&measure).as_bounds().unwrap()).into();
        drawing.draw(bounds, paint().style(paint::Style::Stroke));

        drawing.render()
    }

    #[test]
    fn text_bounds_positioned() {
        let measure = skia::measure::Measure::new();
        let mut drawing = Drawing::new();
        let font = Font::new("", font::Style::default(), font::Size::new(20.0));
        let text = text("Hello World", &font, Point::new(5.0, 5.0));
        drawing.draw(text, paint());
        let bounds: Rect = (*drawing.fast_bounds(&measure).as_bounds().unwrap()).into();
        drawing.draw(bounds, paint().style(paint::Style::Stroke));

        drawing.render()
    }

    #[test]
    fn stack_v() {
        stack_vec(Vector::new(0.0, 1.0));
    }

    #[test]
    fn stack_h() {
        stack_vec(Vector::new(1.0, 0.0));
    }

    fn stack_vec(v: Vector) {
        let measure = skia::measure::Measure::new();
        let stroke_paint_green = paint().style(paint::Style::Stroke).color(0xff00ff00);

        let d1 = hello_world();
        let d2 = hello_world();

        let mut stacked = Drawing::stack(vec![d1, d2], &measure, v);
        let stacked_bounds: Rect = (*stacked.fast_bounds(&measure).as_bounds().unwrap()).into();
        dbg!(&stacked_bounds);
        stacked.draw(stacked_bounds, stroke_paint_green);
        stacked.render()
    }

    fn hello_world() -> Drawing {
        let measure = skia::measure::Measure::new();
        let mut drawing = Drawing::new();
        let font = Font::new("", font::Style::default(), font::Size::new(20.0));
        let text = text("Hello World", &font, None);
        drawing.draw(text, paint());
        let bounds: Rect = (*drawing.fast_bounds(&measure).as_bounds().unwrap()).into();
        drawing.draw(bounds, paint().style(paint::Style::Stroke));
        drawing
    }
}
