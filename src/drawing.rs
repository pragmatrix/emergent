//! Serializable data Structures for unparameterized Drawings
//! Structures here are optimized compact serialization but also for type safety and maximum precision.

// TODO: construction API of these objects need to be separate (perhaps via regular builders and functions?),
//       so that we can make the serialization more compact?

use serde::{Deserialize, Serialize};

//
// Small, Copyable Types.
//

#[allow(non_camel_case_types)]
pub type scalar = f64;

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Point(scalar, scalar);

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Size(scalar, scalar);

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Vector(scalar, scalar);

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Angle(scalar);

// 32-bit ARGB color value.
// TODO: do we really want this? Serialization should be HEX I guess.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Color(u32);

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Scale(scalar, scalar);

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Skew(scalar, scalar);

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Degrees(scalar);

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Radius(scalar);

//
// Larger, Cloneable types.
//

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Rect(Point, Size);

// TODO: Optimize representation for simple cases?
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct RoundedRect(Rect, [Vector; 4]);

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Matrix([scalar; 9]);

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Transformation {
    Translate(Vector),
    Scale(Vector),
    Rotate(Vector),
    Skew(Skew),
    Matrix(Matrix),
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Clip {
    Rect(Rect),
    RoundedRect(RoundedRect),
    Path(Path),
}

// https://developer.android.com/reference/android/graphics/PorterDuff.Mode
// We support 12 alpha composition modes, 5 blending modes, and simple addition for now.
// (these are supported on Android)
// Skia modes unsupported are:
//   Modulate
//   ColorDodge, ColorBurn, SoftLight, HardLight, Difference, Exclusion
//   Hue, Saturation, Color, Luminosity

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum BlendMode {
    Source,
    SourceOver,
    SourceIn,
    SourceAtop,
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

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum PaintStyle {
    Stroke,
    Fill,
    StrokeAndFill,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum StrokeCap {
    Butt,
    Round,
    Square,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum StrokeJoin {
    Miter,
    Round,
    Bevel,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct UseCenter(bool);

// contains Option values to support optimal serialization if values do not diverge from their defaults.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Paint {
    #[serde(skip_serializing_if = "Option::is_none")]
    style: Option<PaintStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<Color>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stroke_width: Option<scalar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stroke_miter: Option<scalar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stroke_cap: Option<StrokeCap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stroke_join: Option<StrokeJoin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    blend_mode: Option<BlendMode>,
}

// TODO: ????
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct ImageId(String);

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum PathFillType {
    Winding,
    EventOdd, // TODO: Inverse?
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum PathDirection {
    CW,
    CCW,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Closed(bool);

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct ForceMoveTo(bool);

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum PathVerb {
    MoveTo(Point),
    LineTo(Point),
    QuadTo(Point, Point),
    ConicTo(Point, Point, scalar),
    CubicTo(Point, Point, Point),
    ArcTo(Rect, Angle, Angle, ForceMoveTo),
    Close,
    // is the direction and / or index too much?
    AddRect(Rect, Option<(PathDirection, usize)>),
    AddOval(Rect, Option<(PathDirection, usize)>),
    AddCircle(Point, Radius, Option<PathDirection>),
    AddArc(Rect, Angle, Angle),
    AddRoundedRect(RoundedRect, Option<PathDirection>),
    AddPolygon(Vec<Point>, Closed),
    // TODO: Do we need to support adding paths?
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Path {
    fill_type: PathFillType,
    matrix: Matrix,
    verbs: Vec<PathVerb>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Shape {
    Point(Point),
    Points(Vec<Point>),
    Line(Point, Point),
    // TODO: should we introduce a type Line?
    Lines(Vec<(Point, Point)>),
    // TODO: should we introduce a type Polygon?
    Polygon(Vec<Point>),
    Rect(Rect),
    Oval(Rect),
    RoundedRect(RoundedRect),
    // TODO: Skia has an optimized function for drawing a rounded rect inside another. Should we support that?
    Circle(Point, Radius),
    Arc(Rect, Angle, Angle, UseCenter),
    Path(Path),
    Image(ImageId),
    ImageRect(ImageId, Option<Rect>, Rect),
    // ImageNine?
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Drawing {
    Transformed(Transformation, Box<Drawing>),
    // TODO: Skia supports ClipOp::Intersect, which I suppose is quite unusual.
    // TODO: Also Skia supports do_anti_alias for clipping.

    // paint independent draw commands:
    Clipped(Clip, Box<Drawing>),
    Color(Color, BlendMode),
    Clear(Color),

    // TODO: think about introducing a WithPaint
    // (which we might be able to optimize).
    // or? a Shapes? where multiple shapes are drawn
    // with the same paint?
    Shape(Shape, Paint),
    Drawings(Vec<Drawing>),
}

#[test]
fn test_serialize() {
    let drawing = Drawing::Shape(
        Shape::Line(Point(10.0, 1.0), Point(11.0, 1.0)),
        Paint {
            style: None,
            color: None,
            stroke_width: None,
            stroke_miter: None,
            stroke_cap: None,
            stroke_join: None,
            blend_mode: None,
        },
    );

    println!("{}", serde_json::to_string(&drawing).unwrap());

    let drawing_in_drawing = Drawing::Clipped(
        Clip::Rect(Rect(Point(10.0, 1.0), Size(10.0, 1.0))),
        Box::new(drawing),
    );

    println!("{}", serde_json::to_string(&drawing_in_drawing).unwrap());
}
