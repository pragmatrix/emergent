//! A captured test and its presentation.

use crate::libtest::TestCapture;
use crate::Msg;
use emergent_drawing::functions::{paint, text};
use emergent_drawing::{font, Drawing, DrawingTarget, Font};
use emergent_presentation::Presentation;
use emergent_presenter::input_processor::Tap;
use emergent_presenter::{
    Direction, IndexMappable, InputProcessor, Item, Reducible, View, ViewBuilder,
};

impl TestCapture {
    pub fn present(&self, mut b: ViewBuilder<Msg>, show_contents: bool) -> View<Msg> {
        let nested = b.scoped(&self.name, |mut b| {
            let header = Item::new(&self.name).map(|mut b, name| {
                let name = name.to_string();
                let presentation = Self::present_header(&name).in_area();
                b.use_input_processor(|| {
                    Tap::new().map(move |_| Some(Msg::ToggleTestcase { name: name.clone() }))
                });
                b.present(presentation)
            });

            if !show_contents {
                return header.reduce(b, ());
            }

            let contents =
                Item::new(&self.output).map(|b, output| b.present(Self::present_output(output)));

            header.extend(&contents).reduce(b, Direction::Column)
        });

        b.wrapped(nested)
    }

    fn present_header(title: &str) -> Presentation {
        let header_font = &Font::new("", font::Style::NORMAL, font::Size::new(20.0));
        let mut drawing = Drawing::new();
        let text = text(title, header_font, None);
        drawing.draw_shape(&text.into(), paint());
        drawing.into()
    }

    fn present_output(output: &str) -> Presentation {
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
    use crate::skia::test_environment::view_builder;
    use emergent_drawing::functions::rect;
    use emergent_drawing::{Drawing, DrawingTarget, Paint, Render, Visualize, RGB};

    #[test]
    fn capture_presentations() {
        let b = view_builder::from_test_environment();

        let output = {
            let mut drawing = Drawing::new();
            drawing.draw(rect((0, 0), (64, 64)), Paint::stroke(0x0023_5689.rgb()));
            format!("> {}", serde_json::to_string(&drawing).unwrap())
        };

        let capture = TestCapture {
            name: "[test-name (open)]".to_string(),
            result: TestResult::Ok(),
            output,
        };

        // TODO: a more direct way to visualize views would be nice, it's a bit confusing to have to clone
        //       support from context before it is consumed.

        let support = b.support().clone();

        let view = capture.present(b, true);
        // TODO: this &* is counter-intuitive too (comes from the Rc wrapper).
        view.into_presentation().visualize(&*support).render();
    }
}
