#[macro_use]
extern crate bitflags;

mod bounds;
pub use bounds::*;

mod canvas;
pub use canvas::*;

mod circle;
pub use circle::*;

mod color;
pub use color::*;

mod degrees;
pub use degrees::*;

mod drawing;
pub use drawing::*;

mod drawing_target;
pub use drawing_target::*;

mod extent;
pub use extent::*;

mod fast_bounds;
pub use fast_bounds::*;

pub mod functions;

pub mod matrix;
pub use matrix::Matrix;

mod outset;
pub use outset::*;

pub mod paint;
pub use paint::Paint;

pub mod path;
pub use path::Path;

mod point;
pub use point::*;

mod radians;
pub use radians::*;

mod radius;
pub use radius::*;

mod rect;
pub use rect::*;

mod rounded_rect;
pub use rounded_rect::*;

mod shape;
pub use shape::*;

mod transform;
pub use transform::*;

mod vector;
pub use vector::*;

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
