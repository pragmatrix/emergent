#[cfg(test)]
mod tests {
    use crate::skia;
    use emergent_drawing::functions::*;
    use emergent_drawing::{
        font, paint, Color, Drawing, DrawingFastBounds, DrawingTarget, Font, Paint, Point, Rect,
        Render,
    };

    #[test]
    fn draw_circle() {
        let mut drawing = Drawing::new();
        let mut paint = &mut Paint::default();
        paint.color = Color::from(0xff0000ff);
        drawing.draw(circle((100, 100), 100), &paint);
        drawing.render();
    }

    #[test]
    fn text_bounds() {
        let measure = skia::measure::Measure::new();
        let mut drawing = Drawing::new();
        let font = Font::new("", font::Style::default(), font::Size::new(20.0));
        let text_paint = paint();
        let text = text(Point::default(), "Hello World", &font);
        drawing.draw(text, &text_paint);
        let stroke_paint = paint().style(paint::Style::Stroke).clone();
        let bounds: Rect = (*drawing.fast_bounds(&measure).as_bounds().unwrap()).into();
        drawing.draw(bounds, &stroke_paint);

        drawing.render()
    }

    #[test]
    fn stack_v() {
        let measure = skia::measure::Measure::new();
        let mut drawing = Drawing::new();
        let font = Font::new("", font::Style::default(), font::Size::new(20.0));
        let text_paint = paint();
        let text = text(Point::default(), "Hello World", &font);
        drawing.draw(text, &text_paint);
        let stroke_paint = paint().style(paint::Style::Stroke).clone();
        let stroke_paint_green = paint()
            .style(paint::Style::Stroke)
            .color(0xff00ff00)
            .clone();
        let bounds: Rect = (*drawing.fast_bounds(&measure).as_bounds().unwrap()).into();
        drawing.draw(bounds, &stroke_paint);

        let d1 = drawing.clone();
        let d2 = drawing.clone();

        let mut stacked = Drawing::stack_v(&measure, vec![d1, d2]);
        let stacked_bounds: Rect = (*stacked.fast_bounds(&measure).as_bounds().unwrap()).into();
        dbg!(&stacked_bounds);
        stacked.draw(stacked_bounds, &stroke_paint_green);

        stacked.render()
    }
}
