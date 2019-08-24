//! Rendering of compiler messages.

use cargo_metadata::CompilerMessage;
use emergent_drawing::{font, functions::*, Drawing, DrawingTarget, Font};

pub trait ToDrawing {
    fn to_drawing(&self) -> Drawing;
}

impl ToDrawing for CompilerMessage {
    fn to_drawing(&self) -> Drawing {
        let mut drawing = Drawing::new();
        let msg = match &self.message.rendered {
            Some(rendered) => &rendered,
            // TODO: test non-rendered messages (are there any?)
            None => &self.message.message,
        };

        // TODO: find some way to define font families and select proper default fonts for each platform.
        let font = Font::new("Fira Code", font::Style::default(), font::Size::new(14.0));
        let text = text(&msg, &font, None);
        drawing.draw(text, paint());
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
    fn draw_message_rendered() {
        let msg = &compiler_messages()[0];
        msg.to_drawing().render()
    }

    #[test]
    fn draw_message_rendered_error_ansi() {
        let msg = &compiler_messages()[2];
        msg.to_drawing().render()
    }

    fn compiler_messages() -> Vec<CompilerMessage> {
        let msgs = MSG.to_owned() + MSG2 + ANSIMSG;
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

    const ANSIMSG : &str = r#"{"reason":"compiler-message","package_id":"emergent 0.1.0 (path+file:///C:/msys/home/armin/dev/emergent)","target":{"kind":["lib"],"crate_types":["lib"],"name":"emergent","src_path":"C:\\msys\\home\\armin\\dev\\emergent\\src/lib/lib.rs","edition":"2018","doctest":true},"message":{"message":"expected one of `!` or `::`, found `fn`","code":null,"level":"error","spans":[{"file_name":"src/lib/lib.rs","byte_start":409,"byte_end":409,"line_start":28,"line_end":28,"column_start":13,"column_end":13,"is_primary":false,"text":[{"text":"    #[test]f","highlight_start":13,"highlight_end":13}],"label":"expected one of `!` or `::` here","suggested_replacement":null,"suggestion_applicability":null,"expansion":null},{"file_name":"src/lib/lib.rs","byte_start":414,"byte_end":416,"line_start":29,"line_end":29,"column_start":5,"column_end":7,"is_primary":true,"text":[{"text":"    fn test_in_mod_capture() {","highlight_start":5,"highlight_end":7}],"label":"unexpected token","suggested_replacement":null,"suggestion_applicability":null,"expansion":null}],"children":[],"rendered":"\u001b[0m\u001b[1m\u001b[38;5;9merror\u001b[0m\u001b[0m\u001b[1m\u001b[38;5;15m: expected one of `!` or `::`, found `fn`\u001b[0m\n\u001b[0m  \u001b[0m\u001b[0m\u001b[1m\u001b[38;5;14m--> \u001b[0m\u001b[0msrc/lib/lib.rs:29:5\u001b[0m\n\u001b[0m   \u001b[0m\u001b[0m\u001b[1m\u001b[38;5;14m|\u001b[0m\n\u001b[0m\u001b[1m\u001b[38;5;14m28\u001b[0m\u001b[0m \u001b[0m\u001b[0m\u001b[1m\u001b[38;5;14m| \u001b[0m\u001b[0m    #[test]f\u001b[0m\n\u001b[0m   \u001b[0m\u001b[0m\u001b[1m\u001b[38;5;14m| \u001b[0m\u001b[0m            \u001b[0m\u001b[0m\u001b[1m\u001b[38;5;14m-\u001b[0m\u001b[0m \u001b[0m\u001b[0m\u001b[1m\u001b[38;5;14mexpected one of `!` or `::` here\u001b[0m\n\u001b[0m\u001b[1m\u001b[38;5;14m29\u001b[0m\u001b[0m \u001b[0m\u001b[0m\u001b[1m\u001b[38;5;14m| \u001b[0m\u001b[0m    fn test_in_mod_capture() {\u001b[0m\n\u001b[0m   \u001b[0m\u001b[0m\u001b[1m\u001b[38;5;14m| \u001b[0m\u001b[0m    \u001b[0m\u001b[0m\u001b[1m\u001b[38;5;9m^^\u001b[0m\u001b[0m \u001b[0m\u001b[0m\u001b[1m\u001b[38;5;9munexpected token\u001b[0m\n\n"}}"#;
}
