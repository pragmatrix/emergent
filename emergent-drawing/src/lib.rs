#[macro_use]
extern crate bitflags;

mod angle;
pub use angle::*;

mod arc;
pub use arc::*;

mod blend_mode;
pub use blend_mode::*;

mod bounds;
pub use bounds::*;

mod canvas;
pub use canvas::*;

pub(crate) mod conic;
pub(crate) use conic::*;

mod circle;
pub use circle::*;

mod clip;
pub use clip::*;

mod color;
pub use color::*;

mod drawing;
pub use drawing::*;

mod drawing_target;
pub use drawing_target::*;

mod extent;
pub use extent::*;

mod fast_bounds;
pub use fast_bounds::*;

pub mod font;
pub use font::Font;

pub mod functions;

mod image;
pub use image::*;

mod line;
pub use line::*;

pub mod matrix;
pub use matrix::Matrix;

pub mod nearly;
pub use nearly::*;

mod outset;
pub use outset::*;

mod oval;
pub use oval::*;

pub mod paint;
pub use paint::Paint;

pub mod path;
pub use path::Path;

mod point;
pub use point::*;

mod polygon;
pub use polygon::*;

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

mod text;
pub use text::*;

mod transform;
pub use transform::*;

mod vector;
pub use vector::*;

//
// Scalar type. f64.
//

#[allow(non_camel_case_types)]
pub type scalar = f64;

pub(crate) trait Scalar {
    const ROOT_2_OVER_2: scalar;
    fn invert(self) -> Self;
}

impl Scalar for scalar {
    const ROOT_2_OVER_2: scalar = std::f64::consts::FRAC_1_SQRT_2;
    fn invert(self) -> Self {
        1.0 / self
    }
}

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

#[cfg(test)]
mod tests {
    use crate::{
        paint, BlendMode, Clip, Color, Draw, Drawing, Line, Paint, Point, Rect, Shape, Vector,
    };

    #[test]
    fn test_serialize() {
        let shapes = Draw::Shapes(
            vec![Shape::Line(Line(Point(10.0, 1.0), Point(11.0, 1.0)))],
            Paint {
                style: paint::Style::Stroke,
                color: Color::from(0xff000000),
                stroke_width: 1.0,
                stroke_miter: 4.0,
                stroke_cap: paint::StrokeCap::Butt,
                stroke_join: paint::StrokeJoin::Miter,
                blend_mode: BlendMode::SourceOver,
            },
        );

        println!("{}", serde_json::to_string(&shapes).unwrap());

        let drawing = Draw::Clipped(
            Clip::Rect(Rect::from((Point(10.0, 1.0), Vector(10.0, 1.0)))),
            Drawing(vec![shapes]),
        );

        println!("{}", serde_json::to_string(&drawing).unwrap());
    }
}
