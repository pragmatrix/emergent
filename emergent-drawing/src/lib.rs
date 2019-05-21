mod canvas;
pub use canvas::*;

mod drawing;
pub use drawing::*;

mod drawing_target;
pub use drawing_target::*;

mod outline;
pub use outline::*;

mod rect;
pub use rect::*;

mod rounded_rect;
pub use rounded_rect::*;

mod shape;
pub use shape::*;

mod vector;
pub use vector::*;

pub trait Render {
    fn render(&self);
}

use std::io;
use std::io::Write;

impl Render for Painting {
    fn render(&self) {
        let rendered = serde_json::to_string(self).unwrap();
        let mut stdout = io::stdout();
        stdout.write(b"> ").unwrap();
        stdout.write_all(rendered.as_bytes()).unwrap();
        stdout.write(b"\n").unwrap();
    }
}

#[derive(Clone, Debug)]
pub struct PaintingCanvas(Painting);

impl PaintingCanvas {
    pub fn new() -> Self {
        PaintingCanvas(Painting::new())
    }

    pub fn render(&self) {
        self.0.render()
    }
}

impl Canvas<Painting> for PaintingCanvas {
    fn target(&mut self) -> &mut Painting {
        &mut self.0
    }
}
