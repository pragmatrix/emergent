use crate::{
    scalar, Arc, Bounds, Circle, FastBounds, Matrix, Oval, Point, Polygon, Rect, RoundedRect,
    Vector,
};
use serde::{Deserialize, Serialize};
use std::iter;

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

impl Default for Direction {
    fn default() -> Self {
        Direction::CW
    }
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

                    points.extend(&r.to_quad());
                    // TODO: this is incorrect, compute the end-point of the arc here.
                    unimplemented!("compute the end-point of the arc here");
                    // current = Some(r.center())
                }
                Verb::Close => {}
                Verb::AddOval(Oval(r), _) => {
                    points.extend(&r.to_quad());
                    // TODO: this is incorrect, use the correct end-point of the rect here.
                    unimplemented!("compute the end-point of the rect");
                    // current = Some(r.center())
                }
                Verb::AddCircle(Circle(p, r), _) => {
                    let sector_size = Vector::from((**r, **r));
                    let r = Rect::from((*p - sector_size, sector_size * 2.0));
                    points.extend(&r.to_quad());
                    // TODO: this is incorrect, use the correct end-point of the rect here.
                    unimplemented!("compute the end-point of the rect");
                    // current = Some(r.center())
                }
                Verb::AddArc(Arc(Oval(r), ..)) => {
                    points.extend(&r.to_quad());
                    // TODO: this is incorrect, compute the end-point of the arc here.
                    unimplemented!("compute the end-point of the rect");
                    // current = Some(r.center())
                }
                Verb::AddRoundedRect(RoundedRect(r, _), _) => {
                    points.extend(&r.to_quad());
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

    //
    // add of complex shapes.
    //

    pub fn add_rect(&mut self, rect: &Rect, dir_start: impl Into<Option<(Direction, usize)>>) {
        let mut iter = rect_point_iterator(rect, dir_start);
        self.reserve_verbs(5)
            .move_to(iter.next().unwrap())
            .line_to(iter.next().unwrap())
            .line_to(iter.next().unwrap())
            .line_to(iter.next().unwrap())
            .close()
    }

    //
    // add primitive verbs.
    //

    pub fn move_to(&mut self, p: impl Into<Point>) -> &mut Self {
        self.add_verb(Verb::MoveTo(p.into()))
    }

    pub fn line_to(&mut self, p: impl Into<Point>) -> &mut Self {
        self.add_verb(Verb::LineTo(p.into()))
    }

    pub fn close(&mut self) {
        self.add_verb(Verb::Close);
    }

    fn add_verb(&mut self, verb: Verb) -> &mut Self {
        self.verbs.push(verb);
        self
    }

    fn reserve_verbs(&mut self, additional: usize) -> &mut Self {
        self.verbs.reserve(additional);
        self
    }
}

fn rect_point_iterator(
    rect: &Rect,
    dir_start: impl Into<Option<(Direction, usize)>>,
) -> impl Iterator<Item = Point> + 'static {
    let (dir, start) = dir_start.into().unwrap_or_default();

    let step = match dir {
        Direction::CW => 1,
        Direction::CCW => -1,
    };

    let mut index = start as isize;
    let rect = rect.clone();

    iter::from_fn(move || {
        let p = match index % 4 {
            0 => rect.left_top(),
            1 => rect.right_top(),
            2 => rect.right_bottom(),
            3 => rect.left_bottom(),
            _ => unreachable!(),
        };

        index += step;
        Some(p)
    })
}

impl FastBounds for Path {
    fn fast_bounds(&self) -> Bounds {
        Bounds::from_points(&self.points()).unwrap()
    }
}
