//! Serializable 2D Graphics Data Structures.
//!
//! TODO: remove all impl Into<Option<T>> and replace None with the unit value if possible.

#[macro_use]
extern crate bitflags;

mod drawing;
pub use drawing::*;

mod drawing_target;
pub use drawing_target::*;

pub mod functions;

pub(crate) mod geometry;
pub use geometry::*;

mod shape;
pub use shape::*;

pub mod simple_layout;

mod toolbox;
pub(crate) use toolbox::*;

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

impl Render for Drawing {
    fn render(&self) {
        use std::io;
        use std::io::Write;

        let rendered = serde_json::to_string(self).unwrap();
        let mut stdout = io::stdout();
        stdout.write(b"> ").unwrap();
        stdout.write_all(rendered.as_bytes()).unwrap();
        stdout.write(b"\n").unwrap();
    }
}

/// A trait used for visuals that can be composed from back to front.
pub trait BackToFront<Composed> {
    fn back_to_front(self) -> Composed;
}

#[cfg(test)]
mod tests {
    use crate::functions::{point, vector};
    use crate::{line::line, paint, BlendMode, Clip, Color, Draw, Drawing, Paint, Rect, Shape};

    #[test]
    fn test_serialize() {
        let shapes = Draw::Shapes(
            vec![Shape::Line(line(point(10.0, 1.0), point(11.0, 1.0)))],
            Paint {
                style: paint::Style::Stroke,
                color: Color::BLACK,
                stroke_width: 1.0,
                stroke_miter: 4.0,
                stroke_cap: paint::StrokeCap::Butt,
                stroke_join: paint::StrokeJoin::Miter,
                blend_mode: BlendMode::SourceOver,
            },
        );

        println!("{}", serde_json::to_string(&shapes).unwrap());

        let drawing = Draw::Clipped(
            Clip::Rect(Rect::from((point(10.0, 1.0), vector(10.0, 1.0)))),
            Drawing::from(vec![shapes]),
        );

        println!("{}", serde_json::to_string(&drawing).unwrap());
    }
}
