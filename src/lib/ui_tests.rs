#[cfg(test)]
mod tests {
    use crate::skia;
    use crate::skia::text::PrimitiveText;
    use emergent_drawing::functions::*;
    use emergent_drawing::{
        font, paint, simple_layout, text, Color, Drawing, DrawingFastBounds, DrawingTarget, Font,
        FromTestEnvironment, Point, Rect, Render, Vector, RGB,
    };

    #[test]
    fn draw_circle() {
        let mut drawing = Drawing::new();
        drawing.draw(circle((32, 32), 32), paint().color(0xff0000ff));
        drawing.render();
    }

    #[test]
    fn text_bounds() {
        bounds_around_text("Bounds around Text").render()
    }

    #[test]
    fn text_multiline_block_bounds() {
        let measure = PrimitiveText::from_test_environment();
        let mut drawing = Drawing::new();
        let font = Font::new("", font::Style::default(), font::Size::new(14.0));
        let text = text::block(&font, None)
            .text("red", 0xff0000.rgb())
            .text(" ", ())
            .text("green", 0x00ff00.rgb())
            .text(" ", ())
            .text("on the first line\n", Color::BLACK)
            .text("and blue on the second line", 0x0000ff.rgb())
            .clone();
        drawing.draw(text, paint());
        let bounds: Rect = (*drawing.fast_bounds(&measure).as_bounds().unwrap()).into();
        drawing.draw(bounds, paint().style(paint::Style::Stroke));

        drawing.render()
    }

    #[test]
    fn text_run_bounds() {
        let measure = PrimitiveText::from_test_environment();
        let mut drawing = Drawing::new();
        let font = Font::new("", font::Style::default(), font::Size::new(14.0));
        let text = text::block(&font, None)
            .text("red", 0xff0000.rgb())
            .text(" ", ())
            .text("green", 0x00ff00.rgb())
            .text(" ", ())
            .text("blue", 0x0000ff.rgb())
            .clone();
        drawing.draw(text, paint());
        let bounds: Rect = (*drawing.fast_bounds(&measure).as_bounds().unwrap()).into();
        drawing.draw(bounds, paint().style(paint::Style::Stroke));

        drawing.render()
    }

    #[test]
    fn text_bounds_positioned() {
        let measure = PrimitiveText::from_test_environment();
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
    fn layout_vertically() {
        stack_vec(Vector::new(0.0, 1.0));
    }

    #[test]
    fn layout_horizontally() {
        stack_vec(Vector::new(1.0, 0.0));
    }

    fn stack_vec(v: Vector) {
        let measure = PrimitiveText::from_test_environment();
        let stroke_paint_green = paint().style(paint::Style::Stroke).color(0xff00ff00);

        let d1 = bounds_around_text("Bounds around Text");
        let d2 = bounds_around_text("Bounds around Text");

        let mut stacked = Drawing::BackToFront(simple_layout::stacked(vec![d1, d2], &measure, v));
        let stacked_bounds: Rect = (*stacked.fast_bounds(&measure).as_bounds().unwrap()).into();
        debug!("{:?}", stacked_bounds);
        stacked.draw(stacked_bounds, stroke_paint_green);
        stacked.render()
    }

    fn bounds_around_text(txt: &str) -> Drawing {
        let measure = PrimitiveText::from_test_environment();
        let mut drawing = Drawing::new();
        let font = Font::new("", font::Style::default(), font::Size::new(14.0));
        let text = text(txt, &font, None);
        drawing.draw(text, paint());
        let bounds: Rect = (*drawing.fast_bounds(&measure).as_bounds().unwrap()).into();
        drawing.draw(bounds, paint().style(paint::Style::Stroke));
        drawing
    }

    #[test]
    fn complex_text_layout() {
        bounds_around_text("The word العربية al-arabiyyah.").render()
    }
}
