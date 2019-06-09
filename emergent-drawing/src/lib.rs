mod canvas;
pub use canvas::*;

mod color;
pub use color::*;

mod drawing;
pub use drawing::*;

mod drawing_target;
pub use drawing_target::*;

pub mod functions;

mod geometry;
pub use geometry::*;

mod outline;
pub use outline::*;

mod paint;
pub use paint::*;

mod rect;
pub use rect::*;

mod rounded_rect;
pub use rounded_rect::*;

mod shape;
pub use shape::*;

pub trait Render {
    fn render(&self);
}

use std::io;
use std::io::Write;

impl Render for Drawing {
    fn render(&self) {
        let rendered = serde_json::to_string(self).unwrap();
        let mut stdout = io::stdout();
        stdout.write(b"> ").unwrap();
        stdout.write_all(rendered.as_bytes()).unwrap();
        stdout.write(b"\n").unwrap();
    }
}

#[derive(Clone, Debug)]
pub struct DrawingCanvas(Drawing);

impl DrawingCanvas {
    pub fn new() -> Self {
        Self(Drawing::new())
    }

    pub fn render(&self) {
        self.0.render()
    }
}

impl Canvas<Drawing> for DrawingCanvas {
    fn target(&mut self) -> &mut Drawing {
        &mut self.0
    }
}
