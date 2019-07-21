//! Serializable data Structures for unparameterized Drawings
//! Structures here are optimized compact serialization but also for type safety and maximum precision.

use crate::{Paint, Path, Point, Radius, Rect, RoundedRect, Transform};
use serde::{Deserialize, Serialize};

pub mod font;
pub use font::Font;

#[derive(Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct Drawing(pub Vec<Draw>);

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Draw {
    /// Fill the current clipping area with the given paint and blend mode.
    Paint(Paint, BlendMode),

    /// Draw a number of shapes with the same paint.
    Shapes(Vec<Shape>, Paint),

    // TODO: Skia supports ClipOp::Difference, which I suppose is quite unusual.
    // TODO: Also Skia supports do_anti_alias for clipping.
    /// Intersect the current clip with the given Clip and draw the nested drawing.
    Clipped(Clip, Drawing),

    /// Draw a drawing transformed with the current matrix.
    Transformed(Transform, Drawing),
}

//
// Shapes
//

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Shape {
    Point(Point),
    Line(Line),
    Polygon(Polygon),
    Rect(Rect),
    Oval(Oval),
    RoundedRect(RoundedRect),
    // TODO: Skia has an optimized function for drawing a rounded rect inside another. Should we support that?
    Circle(Circle),
    Arc(Arc),
    Path(Path),
    Image(ImageId, Option<Rect>, Rect),
    // ImageNine?
    Text(Text),
}

//
// Elementary Shapes
//

/// A line defined by two points.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Line(pub Point, pub Point);

/// A circle at a center point with a radius.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Circle(pub Point, pub Radius);

/// An Oval, described by a Rect.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Oval(pub Rect);

/// A Polygon, closed when used as a shape, open when added to a path.
// TODO: should a minimum number of ponts be constrained
//       (this is critical for computing bounds())
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Polygon(pub Vec<Point>);

// TODO: not sure what that means, verify relation to Path / Shape.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct UseCenter(pub bool);

// An Arc, described by an oval, start angle, and sweep angle.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Arc(pub Oval, pub Angle, pub Angle, pub UseCenter);

/// Text, described by a location, a string, and the font.
// TODO: can we share fonts?
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Text(pub Point, pub String, pub Font);

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Clip {
    Rect(Rect),
    RoundedRect(RoundedRect),
    Path(Path),
}

//
// Geometric Primitives.
//

#[allow(non_camel_case_types)]
pub type scalar = f64;

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct Angle(pub scalar);

// 32-bit ARGB color value.
// TODO: do we really want this? Serialization should be HEX I guess.
// Also: what about a decent color type, say 4 f32 values, may be both?
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Color(pub u32);

// https://developer.android.com/reference/android/graphics/PorterDuff.Mode
// We support 12 alpha composition modes, 5 blending modes, and simple addition for now.
// (these are supported on Android)
// Skia modes unsupported are:
//   Modulate
//   ColorDodge, ColorBurn, SoftLight, HardLight, Difference, Exclusion
//   Hue, Saturation, Color, Luminosity

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
#[repr(usize)]
pub enum BlendMode {
    Source,
    SourceOver,
    SourceIn,
    SourceATop,
    Destination,
    DestinationOver,
    DestinationIn,
    DestinationAtop,
    Clear,
    SourceOut,
    DestinationOut,
    ExclusiveOr,

    Darken,
    Lighten,
    Multiply,
    Screen,
    Overlay,

    Add,
}

// TODO: What should an Image be / refer to? A file, a http:// URL?
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct ImageId(String);

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
