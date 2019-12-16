//! A captured test and its presentation.

use crate::libtest::TestCapture;
use emergent_drawing::functions::{paint, text};
use emergent_drawing::{font, Drawing, DrawingTarget, Font};
use emergent_presentation::Scope;
use emergent_presenter::{Direction, Presenter};

impl TestCapture {
    pub fn present(&self, p: &mut Presenter, show_contents: bool) {
        p.scoped(&self.name, |p| {
            p.stack_f(
                Direction::Column,
                &[&|p| p.area(|p| p.draw(self.header())), &|p| {
                    if show_contents {
                        p.draw(self.output());
                    }
                }],
            )
        })
    }

    fn header(&self) -> Drawing {
        let header_font = &Font::new("", font::Style::NORMAL, font::Size::new(20.0));
        let mut drawing = Drawing::new();
        let text = text(&self.name, header_font, None);
        drawing.draw_shape(&text.into(), paint());
        drawing
    }

    fn output(&self) -> Drawing {
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
    use crate::skia::test_environment::presenter;
    use emergent_drawing::functions::rect;
    use emergent_drawing::{Drawing, DrawingTarget, Paint, Render, Visualize, RGB};

    #[test]
    fn capture_presentations() {
        let mut presenter = presenter::from_test_environment();

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

        capture.present(&mut presenter, true);

        presenter.take_presentation().visualize(&presenter).render();
    }
}
