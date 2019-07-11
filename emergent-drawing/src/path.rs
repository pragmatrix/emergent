use crate::{
    scalar, Arc, Bounds, Circle, FastBounds, Matrix, Oval, Point, Polygon, Radius, Rect,
    RoundedRect, Vector,
};
use serde::{Deserialize, Serialize};

//
// Path
//

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Path {
    fill_type: FillType,
    // TODO: do we need a matrix here, isn't that redundant given that we
    //       can transform the Path in the context where it is used.
    matrix: Matrix,
    verbs: Vec<Verb>,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum FillType {
    Winding,
    EvenOdd, // TODO: Inverse?
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Direction {
    CW,
    CCW,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct ForceMoveTo(pub bool);

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Verb {
    MoveTo(Point),
    LineTo(Point),
    QuadTo(Point, Point),
    ConicTo(Point, Point, scalar),
    CubicTo(Point, Point, Point),
    ArcTo(Arc, ForceMoveTo),
    Close,
    // is the direction and / or index too much?
    AddRect(Rect, Option<(Direction, usize)>),
    AddOval(Oval, Option<(Direction, usize)>),
    AddCircle(Circle, Option<Direction>),
    AddArc(Arc),
    AddRoundedRect(RoundedRect, Option<Direction>),
    AddOpenPolygon(Polygon),
    // TODO: Do we need to support adding paths?
}

// TODO: add path combinators!

impl Path {
    // TODO: complete the implementation.
    pub fn points(&self) -> Vec<Point> {
        let mut current: Option<Point> = None;

        // TODO: we don't need to store the points here and could do boundary
        // computation while we iterate through the path ops.
        let mut points = Vec::new();

        for v in self.verbs.iter() {
            match v {
                Verb::MoveTo(p) => {
                    points.push(*p);
                    current = Some(*p)
                }
                Verb::LineTo(p2) => {
                    let p1 = current.unwrap_or_default();
                    points.extend(&[p1, *p2]);
                    current = Some(*p2);
                }
                Verb::QuadTo(p2, p3) => {
                    let p1 = current.unwrap_or_default();
                    points.extend(&[p1, *p2, *p3]);
                    current = Some(*p3);
                }
                Verb::ConicTo(p2, p3, _) => {
                    let p1 = current.unwrap_or_default();
                    points.extend(&[p1, *p2, *p3]);
                    current = Some(*p3);
                }
                Verb::CubicTo(p2, p3, p4) => {
                    let p1 = current.unwrap_or_default();
                    points.extend(&[p1, *p2, *p3, *p4]);
                    current = Some(*p4);
                }
                Verb::ArcTo(Arc(Oval(r), _, _, _), ForceMoveTo(fmt)) => {
                    // TODO: clarify exactly what ForceMoveTo means.
                    if *fmt {
                        let current = current.unwrap_or_default();
                        points.push(current);
                    };

                    points.extend(&r.points());
                    // TODO: this is incorrect, compute the end-point of the arc here.
                    unimplemented!("compute the end-point of the arc here");
                    // current = Some(r.center())
                }
                Verb::Close => {}
                Verb::AddRect(r, _) => {
                    points.extend(&r.points());
                    // TODO: this is incorrect, use the correct end-point of the rect here.
                    unimplemented!("compute the end-point of the rect");
                    // current = Some(r.center())
                }
                Verb::AddOval(Oval(r), _) => {
                    points.extend(&r.points());
                    // TODO: this is incorrect, use the correct end-point of the rect here.
                    unimplemented!("compute the end-point of the rect");
                    // current = Some(r.center())
                }
                Verb::AddCircle(Circle(p, Radius(r)), _) => {
                    let sector_size = Vector::from((*r, *r));
                    let r = Rect::from((*p - sector_size, sector_size * 2.0));
                    points.extend(&r.points());
                    // TODO: this is incorrect, use the correct end-point of the rect here.
                    unimplemented!("compute the end-point of the rect");
                    // current = Some(r.center())
                }
                Verb::AddArc(Arc(Oval(r), ..)) => {
                    points.extend(&r.points());
                    // TODO: this is incorrect, compute the end-point of the arc here.
                    unimplemented!("compute the end-point of the rect");
                    // current = Some(r.center())
                }
                Verb::AddRoundedRect(RoundedRect(r, _), _) => {
                    points.extend(&r.points());
                    current = Some(r.center())
                }
                Verb::AddOpenPolygon(Polygon(pts)) => {
                    points.extend(pts);
                    current = pts.last().cloned().or(current);
                }
            }
        }

        points
    }
}

impl FastBounds for Path {
    fn fast_bounds(&self) -> Bounds {
        Bounds::from_points(&self.points()).unwrap()
    }
}
