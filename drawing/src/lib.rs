//! Serializable 2D Graphics Data Structures.
//!
//! TODO: remove all impl Into<Option<T>> and replace None with the unit value if possible.

#[macro_use]
extern crate bitflags;

mod drawing;
pub use drawing::*;

mod fast_bounds;
pub use fast_bounds::*;

mod drawing_target;
pub use drawing_target::*;

pub mod functions;

pub(crate) mod geometry;
pub use geometry::*;

mod shape;
pub use shape::*;

pub mod simple_layout;

pub(crate) mod skia_core;

mod toolbox;
pub(crate) use toolbox::*;

mod visualize;
pub use visualize::*;

//
// Scalar type. f64.
//

#[allow(non_camel_case_types)]
pub type scalar = f64;

pub(crate) trait Scalar {
    const ROOT_2_OVER_2: scalar;
    fn invert(self) -> Self;
    fn square(self) -> Self;
}

impl Scalar for scalar {
    const ROOT_2_OVER_2: scalar = std::f64::consts::FRAC_1_SQRT_2;
    fn invert(self) -> Self {
        1.0 / self
    }
    fn square(self) -> Self {
        self * self
    }
}

pub trait Contains<Shape> {
    fn contains(&self, what: Shape) -> bool;
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
    use crate::functions::{line, point, rect, vector};
    use crate::{paint, BlendMode, Clipped, Color, IntoDrawing, IntoShape, Paint};

    #[test]
    fn test_serialize() {
        let line = line(point(10.0, 1.0), point(11.0, 1.0))
            .into_shape()
            .into_drawing()
            .with_paint(Paint {
                style: paint::Style::Stroke,
                color: Color::BLACK,
                width: 1.0,
                miter: 4.0,
                cap: paint::Cap::Butt,
                join: paint::Join::Miter,
                blend_mode: BlendMode::SourceOver,
            });

        println!("{}", serde_json::to_string(&line).unwrap());

        let clipped_drawing = line.clipped(rect(point(10.0, 1.0), vector(10.0, 1.0)));

        println!("{}", serde_json::to_string(&clipped_drawing).unwrap());
    }
}

/// A trait for support to pull configuration data from the test environment.
/// TODO: hide behind cfg(test) but consider IDE support.
pub trait FromTestEnvironment {
    fn from_test_environment() -> Self;
}
