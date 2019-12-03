//! A captured test and its presentation.

use crate::libtest::TestCapture;
use emergent_drawing::functions::{paint, text};
use emergent_drawing::simple_layout::SimpleLayout;
use emergent_drawing::{font, Drawing, DrawingTarget, Font};
use emergent_presentation::{IntoPresentation, Presentation, Scope};
use emergent_presenter::Presenter;

impl TestCapture {
    pub fn present(
        &self,
        scope: Scope,
        show_contents: bool,
        presenter: &Presenter,
    ) -> Presentation {
        let header = self.present_header(scope);
        if !show_contents {
            return header;
        }
        let output = self.draw_output();

        Presentation::BackToFront(Presentation::layout_vertically(
            vec![header, output.into()],
            presenter,
        ))
    }

    fn present_header(&self, scope: Scope) -> Presentation {
        let header_font = &Font::new("", font::Style::NORMAL, font::Size::new(20.0));
        let mut drawing = Drawing::new();
        let text = text(&self.name, header_font, None);
        drawing.draw_shape(&text.into(), paint());
        drawing.into_presentation().in_area(scope)
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
    use emergent_drawing::functions::rect;
    use emergent_drawing::{Drawing, DrawingTarget, Paint, Render, Visualize, RGB};

    #[test]
    fn capture_presentations() {
        let presenter = crate::skia::test_environment::presenter::from_test_environment();

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
            .present(0.into(), true, &presenter)
            .visualize(&presenter)
            .render();
    }
}
