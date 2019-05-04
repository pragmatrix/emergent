//! Serializable data Structures for unparameterized Drawings
//! Structures here are optimized compact serialization but also for type safety and maximum precision.

// TODO: construction API of these objects need to be separate (perhaps via regular builders and functions?),
//       so that we can make the serialization more compact?

use serde::{Deserialize, Serialize};

//
// Geometric Primitives.
//

#[allow(non_camel_case_types)]
pub type scalar = f64;

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Point(pub scalar, pub scalar);

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Size(pub scalar, pub scalar);

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Vector(pub scalar, pub scalar);

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Angle(pub scalar);

// 32-bit ARGB color value.
// TODO: do we really want this? Serialization should be HEX I guess.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Color(u32);

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Scale(pub scalar, pub scalar);

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Skew(pub scalar, pub scalar);

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Degrees(scalar);

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Radius(pub scalar);

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Matrix([scalar; 9]);

//
// Elementary Shapes
//

/// A line defined by two points.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Line(pub Point, pub Point);

/// A rectangle, defined by a point and a size.
// TODO: should we separate Rect as a mathematic tool from
// the Rectangle Shape geometric form?
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Rect(pub Point, pub Size);

/// A rounded rectangle.
// TODO: Optimize representation for simple cases?
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct RoundedRect(Rect, [Vector; 4]);

/// A circle at a center point with a radius.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Circle(pub Point, pub Radius);

/// An Oval, described by a Rect.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Oval(pub Rect);

/// Line segments, an open polygon.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct LineSegments(pub Vec<Point>);

/// A Polygon, always closed.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Polygon(pub Vec<Point>);

// TODO: not sure what that means, verify relation to Path / Shape.
#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct UseCenter(pub bool);

// An Arc, described by an Avove, start angle, and sweep angle.
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Arc(pub Oval, pub Angle, pub Angle, pub UseCenter);

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

// contains Option values to support optimal serialization if values do not diverge from their defaults.
#[derive(Clone, Serialize, Deserialize, PartialEq, Default, Debug)]
pub struct Paint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<PaintStyle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<Color>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<scalar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_miter: Option<scalar>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_cap: Option<StrokeCap>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_join: Option<StrokeJoin>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blend_mode: Option<BlendMode>,
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
pub struct ForceMoveTo(pub bool);

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum PathVerb {
    MoveTo(Point),
    LineTo(Point),
    QuadTo(Point, Point),
    ConicTo(Point, Point, scalar),
    CubicTo(Point, Point, Point),
    ArcTo(Arc, ForceMoveTo),
    Close,
    // is the direction and / or index too much?
    AddRect(Rect, Option<(PathDirection, usize)>),
    AddOval(Oval, Option<(PathDirection, usize)>),
    AddCircle(Circle, Option<PathDirection>),
    AddArc(Arc),
    AddRoundedRect(RoundedRect, Option<PathDirection>),
    AddLineSegments(LineSegments),
    AddPolygon(Polygon),
    // TODO: Do we need to support adding paths?
}

// TODO: path combinators!

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Path {
    fill_type: PathFillType,
    matrix: Matrix,
    verbs: Vec<PathVerb>,
}

// TODO: can't we _just_ use a Trait here?
#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Shape {
    Point(Point),
    // TODO: unwrap to multiple Shapes and optimize that later by comparing Paints?
    Points(Vec<Point>),
    Line(Line),
    // TODO: unwrap to multiple Shapes and optimize that later by comparing Paints?
    Lines(Vec<Line>),
    LineSegments(LineSegments),
    Polygon(Polygon),
    Rect(Rect),
    Oval(Oval),
    RoundedRect(RoundedRect),
    // TODO: Skia has an optimized function for drawing a rounded rect inside another. Should we support that?
    Circle(Circle),
    Arc(Arc),
    Path(Path),
    Image(ImageId),
    ImageRect(ImageId, Option<Rect>, Rect),
    // ImageNine?
}

pub type Painting = Vec<Drawing>;

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Drawing {
    /// Fill that current area with the given paint.
    Fill(Paint, BlendMode),

    /// Draw a group of shapes with the same paint.
    Draw(Vec<Shape>, Paint),

    /// A nested painting, save the current matrix and clip,
    /// and restores it afterwards.
    Paint(Painting),

    // TODO: Skia supports ClipOp::Difference, which I suppose is quite unusual.
    // TODO: Also Skia supports do_anti_alias for clipping.
    /// Intersect the current clip with the given Clip.
    Clip(Clip),

    /// Transform the current matrix.
    Transform(Transformation),
}

//
// Shape From implementations.
//

impl From<Point> for Shape {
    fn from(point: Point) -> Self {
        Shape::Point(point)
    }
}

impl From<Line> for Shape {
    fn from(line: Line) -> Self {
        Shape::Line(line)
    }
}

impl From<LineSegments> for Shape {
    fn from(line_segments: LineSegments) -> Self {
        Shape::LineSegments(line_segments)
    }
}

impl From<Polygon> for Shape {
    fn from(polygon: Polygon) -> Self {
        Shape::Polygon(polygon)
    }
}

impl From<Rect> for Shape {
    fn from(rect: Rect) -> Self {
        Shape::Rect(rect)
    }
}

impl From<Oval> for Shape {
    fn from(oval: Oval) -> Self {
        Shape::Oval(oval)
    }
}

impl From<RoundedRect> for Shape {
    fn from(rounded_rect: RoundedRect) -> Self {
        Shape::RoundedRect(rounded_rect)
    }
}

impl From<Circle> for Shape {
    fn from(circle: Circle) -> Self {
        Shape::Circle(circle)
    }
}

impl From<Arc> for Shape {
    fn from(arc: Arc) -> Self {
        Shape::Arc(arc)
    }
}

impl From<Path> for Shape {
    fn from(path: Path) -> Self {
        Shape::Path(path)
    }
}

// TODO: ImageId / ImageRect

#[test]
fn test_serialize() {
    let drawing = Drawing::Draw(
        vec![Shape::Line(Line(Point(10.0, 1.0), Point(11.0, 1.0)))],
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

    let drawing = Drawing::Clip(Clip::Rect(Rect(Point(10.0, 1.0), Size(10.0, 1.0))));

    println!("{}", serde_json::to_string(&drawing).unwrap());
}
