//! A captured test and its presentation.

use crate::libtest::TestCapture;
use crate::Msg;
use emergent_drawing::functions::{paint, text};
use emergent_drawing::{font, Drawing, DrawingTarget, Font};
use emergent_presenter::recognizer::TapRecognizer;
use emergent_presenter::{Context, Direction, IndexMappable, Item, Reducible, View};

impl TestCapture {
    pub fn present(&self, c: &mut Context<Msg>, show_contents: bool) -> View<Msg> {
        c.nested(&self.name, |c| {
            let header = Item::new(&self.name).map(|_, name| {
                let name = name.to_string();
                Self::view_header(&name)
                    .in_area()
                    .with_recognizer(TapRecognizer::new(move || Msg::ToggleTestcase {
                        name: name.clone(),
                    }))
            });

            if !show_contents {
                return header.reduce(c, ());
            }

            let contents =
                Item::new(&self.output).map(|_, output| Self::view_output(output).into());

            header.extend(&contents).reduce(c, Direction::Column)
        })
    }

    fn view_header(title: &str) -> View<Msg> {
        let ref header_font = Font::new("", font::Style::NORMAL, font::Size::new(20.0));
        let mut drawing = Drawing::new();
        let text = text(title, header_font, None);
        drawing.draw_shape(&text.into(), paint());
        drawing.into()
    }

    fn view_output(output: &str) -> View<Msg> {
        // TODO: render invalid output as text and mark it appropriately
        if !output.starts_with("> ") {
            return Drawing::new().into();
        };

        // TODO: handle parse errors:
        let drawing: Drawing = serde_json::from_str(&output[2..]).unwrap();
        drawing.into()
    }
}

#[cfg(test)]
mod tests {
    use crate::libtest::{TestCapture, TestResult};
    use crate::skia::test_environment::context;
    use emergent_drawing::functions::rect;
    use emergent_drawing::{Drawing, DrawingTarget, Paint, Render, Visualize, RGB};

    #[test]
    fn capture_presentations() {
        let mut context = context::from_test_environment();

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

        let view = capture.present(&mut context, true);
        view.into_presentation().visualize(&context).render();
    }
}
