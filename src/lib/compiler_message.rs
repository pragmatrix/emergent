//! Rendering of compiler messages.

use cargo_metadata::CompilerMessage;
use emergent_drawing::{font, functions::*, Drawing, DrawingTarget, Font};

pub trait ToDrawing {
    fn to_drawing(&self) -> Drawing;
}

impl ToDrawing for CompilerMessage {
    fn to_drawing(&self) -> Drawing {
        let message = format!("{:?}", self.message);
        let mut drawing = Drawing::new();
        let font = Font::new("", font::Style::default(), font::Size::new(16.0));
        let text = text(&message, &font, None);
        let paint = paint();
        drawing.draw(text, &paint);
        drawing
    }
}

#[cfg(test)]
mod tests {
    use crate::compiler_message::ToDrawing;
    use cargo_metadata::{CompilerMessage, Message};
    use emergent_drawing::Render;
    use std::io::Cursor;

    #[test]
    fn draw_error_rendered() {
        let msg = &compiler_messages()[0];
        msg.to_drawing().render()
    }

    #[test]
    fn draw_error_single_line() {
        let msg = &compiler_messages()[0];
        msg.to_drawing().render()
    }

    fn compiler_messages() -> Vec<CompilerMessage> {
        let msgs = MSG.to_owned() + MSG2;
        let msgs = cargo_metadata::parse_messages(Cursor::new(msgs));
        msgs.filter_map(|msg| {
            if let Ok(Message::CompilerMessage(msg)) = msg {
                Some(msg)
            } else {
                None
            }
        })
        .collect()
    }

    const MSG : &str = r#"{"reason":"compiler-message","package_id":"emergent 0.1.0 (path+file:///C:/emergent)","target":{"kind":["lib"],"crate_types":["lib"],"name":"emergent","src_path":"C:\\emergent\\src/lib/lib.rs","edition":"2018"},"message":{"message":"expected one of `!` or `::`, found `#`","code":null,"level":"error","spans":[{"file_name":"src/lib/lib.rs","byte_start":451,"byte_end":451,"line_start":27,"line_end":27,"column_start":6,"column_end":6,"is_primary":false,"text":[{"text":"    x","highlight_start":6,"highlight_end":6}],"label":"expected one of `!` or `::` here","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"src/lib/lib.rs","byte_start":734,"byte_end":735,"line_start":37,"line_end":37,"column_start":5,"column_end":6,"is_primary":true,"text":[{"text":"    #[test]","highlight_start":5,"highlight_end":6}],"label":"unexpected token","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"error: expected one of `!` or `::`, found `#`\n  --> src/lib/lib.rs:37:5\n   |\n27 |     x\n   |      - expected one of `!` or `::` here\n...\n37 |     #[test]\n   |     ^ unexpected token\n\n"}}"#;
    const MSG2 : &str = r#"{"reason":"compiler-message","package_id":"emergent 0.1.0 (path+file:///C:/emergent)","target":{"kind":["lib"],"crate_types":["lib"],"name":"emergent","src_path":"C:\\emergent\\src/lib/lib.rs","edition":"2018"},"message":{"message":"aborting due to previous error","code":null,"level":"error","spans":[],"children":[],"rendered":"error: aborting due to previous error\n\n"}}"#;
}
