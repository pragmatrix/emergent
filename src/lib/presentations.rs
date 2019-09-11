//! A captured test and its presentation.

use crate::libtest::TestCapture;
use emergent_drawing::functions::{paint, text};
use emergent_drawing::simple_layout::SimpleLayout;
use emergent_drawing::{font, BackToFront, Drawing, DrawingTarget, Font, MeasureText};
use emergent_presentation::{Area, Present, Presentation};

impl TestCapture {
    pub fn present(
        &self,
        header_area: Area,
        show_contents: bool,
        measure: &dyn MeasureText,
    ) -> Presentation {
        let header = self.present_header(header_area);
        if !show_contents {
            return header;
        }
        let output = self.draw_output().present();

        Presentation::layout_vertically(vec![header, output], measure).back_to_front()
    }

    fn present_header(&self, area: Area) -> Presentation {
        let header_font = &Font::new("", font::Style::NORMAL, font::Size::new(20.0));
        let mut drawing = Drawing::new();
        let text = text(&self.name, header_font, None);
        drawing.draw_shape(&text.into(), paint());
        drawing.present().in_area(area)
    }

    fn draw_output(&self) -> Drawing {
        // TODO: render invalid output as text and mark it appropriately
        if !self.output.starts_with("> ") {
            return Drawing::new();
        };

        // TODO: handle parse errors:
        serde_json::from_str(&self.output[2..]).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::libtest::{TestCapture, TestResult};
    use crate::skia::text::PrimitiveText;
    use emergent_drawing::functions::rect;
    use emergent_drawing::{Drawing, DrawingTarget, Paint, Render, Visualize, RGB};
    use emergent_presentation::Area;

    #[test]
    fn capture_presentations() {
        let measure = PrimitiveText::default();

        let output = {
            let mut drawing = Drawing::new();
            drawing.draw(rect((0, 0), (64, 64)), Paint::stroke(0x235689.rgb()));
            format!("> {}", serde_json::to_string(&drawing).unwrap())
        };

        let capture = TestCapture {
            name: "[test-name (open)]".to_string(),
            result: TestResult::Ok(),
            output,
        };

        capture
            .present(Area::Named("header".into()), true, &measure)
            .visualize(&measure)
            .render();
    }
}
