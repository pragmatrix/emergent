#[cfg(test)]
mod tests {
    use crate::skia;
    use emergent_drawing::functions::*;
    use emergent_drawing::{
        font, paint, text, Color, Drawing, DrawingFastBounds, DrawingTarget, Font, Point, Rect,
        Render, Vector, RGB,
    };

    #[test]
    fn draw_circle() {
        let mut drawing = Drawing::new();
        drawing.draw(circle((32, 32), 32), paint().color(0xff0000ff));
        drawing.render();
    }

    #[test]
    fn text_bounds() {
        let measure = skia::measure::Measure::new();
        let mut drawing = Drawing::new();
        let font = Font::new("", font::Style::default(), font::Size::new(14.0));
        let text = text("Bounds around Text", &font, None);
        drawing.draw(text, paint());
        let bounds: Rect = (*drawing.fast_bounds(&measure).as_bounds().unwrap()).into();
        drawing.draw(bounds, paint().style(paint::Style::Stroke));

        drawing.render()
    }

    #[test]
    fn text_block_bounds() {
        let measure = skia::measure::Measure::new();
        let mut drawing = Drawing::new();
        let font = Font::new("", font::Style::default(), font::Size::new(14.0));
        let text = text::block(&font, None)
            .text("red", 0xff0000.rgb())
            .text(" ", ())
            .text("green", 0x00ff00.rgb())
            .text(" ", ())
            .text("on the first line", Color::BLACK)
            .eol()
            .text("and blue on the second line", 0x0000ff.rgb())
            .clone();
        drawing.draw(text, paint());
        let bounds: Rect = (*drawing.fast_bounds(&measure).as_bounds().unwrap()).into();
        drawing.draw(bounds, paint().style(paint::Style::Stroke));

        drawing.render()
    }

    #[test]
    fn text_bounds_positioned() {
        let measure = skia::measure::Measure::new();
        let mut drawing = Drawing::new();
        let font = Font::new("", font::Style::default(), font::Size::new(14.0));
        let text = text(
            "Text positioned at (5,5) should appear unpositioned when rendered in a testcase",
            &font,
            Point::new(5.0, 5.0),
        );
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

        let d1 = bounds_around_text();
        let d2 = bounds_around_text();

        let mut stacked = Drawing::stack(vec![d1, d2], &measure, v);
        let stacked_bounds: Rect = (*stacked.fast_bounds(&measure).as_bounds().unwrap()).into();
        dbg!(&stacked_bounds);
        stacked.draw(stacked_bounds, stroke_paint_green);
        stacked.render()
    }

    fn bounds_around_text() -> Drawing {
        let measure = skia::measure::Measure::new();
        let mut drawing = Drawing::new();
        let font = Font::new("", font::Style::default(), font::Size::new(14.0));
        let text = text("Bounds around Text", &font, None);
        drawing.draw(text, paint());
        let bounds: Rect = (*drawing.fast_bounds(&measure).as_bounds().unwrap()).into();
        drawing.draw(bounds, paint().style(paint::Style::Stroke));
        drawing
    }
}
