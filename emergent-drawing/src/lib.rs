mod canvas;
pub use canvas::*;

mod drawing;
pub use drawing::*;

mod drawing_target;
pub use drawing_target::*;

pub trait Render {
    fn render(&self);
}

use std::io;
use std::io::Write;

impl Render for Painting {
    fn render(&self) {
        let rendered = serde_json::to_string(self).unwrap();
        let mut stdout = io::stdout();
        stdout.write(b"> ");
        stdout.write_all(rendered.as_bytes()).unwrap();
        stdout.write(b"\n");
    }
}
