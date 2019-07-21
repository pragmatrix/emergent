use crate::{
    scalar, Arc, Bounds, Circle, FastBounds, Matrix, Oval, Point, Polygon, Rect, RoundedRect,
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

    AddArc(Arc),
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
                Verb::AddArc(Arc(Oval(r), ..)) => {
                    points.extend(&r.to_quad());
                    // TODO: this is incorrect, compute the end-point of the arc here.
                    unimplemented!("compute the end-point of the rect");
                    // current = Some(r.center())
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

    pub fn add_rounded_rect(
        &mut self,
        rr: &RoundedRect,
        dir_start: impl Into<Option<(Direction, usize)>>,
    ) {
        let (dir, start) = dir_start.into().unwrap_or_default();

        // CW: odd indices
        // CCW: even indices
        let starts_with_conic = ((start & 1) != 0) == (dir == Direction::CW);
        const WEIGHT: f64 = std::f64::consts::FRAC_1_SQRT_2;

        let mut rrect_iter = rounded_rect_point_iterator(rr, (dir, start));
        let rect_start_index = start / 2 + (if dir == Direction::CW { 0 } else { 1 });
        let mut rect_iter = rect_point_iterator(rr.rect(), (dir, rect_start_index));

        self.reserve_verbs(if starts_with_conic { 9 } else { 10 })
            .move_to(rrect_iter.next().unwrap());

        if starts_with_conic {
            for _ in 0..=2 {
                self.conic_to(
                    rect_iter.next().unwrap(),
                    rrect_iter.next().unwrap(),
                    WEIGHT,
                )
                .line_to(rrect_iter.next().unwrap());
            }
            self.conic_to(
                rect_iter.next().unwrap(),
                rrect_iter.next().unwrap(),
                WEIGHT,
            );
        } else {
            for _ in 0..=3 {
                self.line_to(rrect_iter.next().unwrap()).conic_to(
                    rect_iter.next().unwrap(),
                    rrect_iter.next().unwrap(),
                    WEIGHT,
                );
            }
        }

        self.close();
    }

    pub fn add_oval(&mut self, oval: &Oval, dir_start: impl Into<Option<(Direction, usize)>>) {
        self.reserve_verbs(6);
        const WEIGHT: f64 = std::f64::consts::FRAC_1_SQRT_2;

        let (dir, start) = dir_start.into().unwrap_or_default();

        let mut oval_iter = oval_point_iterator(&oval.0, (dir, start));
        let mut rect_iter = rect_point_iterator(
            &oval.0,
            (dir, start + if dir == Direction::CW { 0 } else { 1 }),
        );

        self.move_to(oval_iter.next().unwrap());
        for _ in 0..=3 {
            self.conic_to(rect_iter.next().unwrap(), oval_iter.next().unwrap(), WEIGHT);
        }
        self.close();
    }

    pub fn add_circle(&mut self, circle: &Circle, dir: impl Into<Option<Direction>>) {
        let dir = dir.into().unwrap_or_default();
        // TODO: does it make sense here to support a starting index?
        self.add_oval(&circle.to_oval(), (dir, 0))
    }

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

    pub fn conic_to(
        &mut self,
        p1: impl Into<Point>,
        p2: impl Into<Point>,
        weight: scalar,
    ) -> &mut Self {
        if !(weight > 0.0) {
            return self.line_to(p2);
        }

        if !weight.is_finite() {
            return self.line_to(p1).line_to(p2);
        }

        if weight == 1.0 {
            return self.quad_to(p1, p2);
        }

        self.add_verb(Verb::ConicTo(p1.into(), p2.into(), weight))
    }

    pub fn quad_to(&mut self, p1: impl Into<Point>, p2: impl Into<Point>) -> &mut Self {
        self.add_verb(Verb::QuadTo(p1.into(), p2.into()))
    }

    pub fn line_to(&mut self, p: impl Into<Point>) -> &mut Self {
        self.add_verb(Verb::LineTo(p.into()))
    }

    pub fn move_to(&mut self, p: impl Into<Point>) -> &mut Self {
        self.add_verb(Verb::MoveTo(p.into()))
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
) -> impl Iterator<Item = Point> {
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

fn rounded_rect_point_iterator(
    rrect: &RoundedRect,
    dir_start: impl Into<Option<(Direction, usize)>>,
) -> impl Iterator<Item = Point> {
    let (dir, start) = dir_start.into().unwrap_or_default();

    let step = match dir {
        Direction::CW => 1,
        Direction::CCW => -1,
    };

    let mut index = start as isize;
    let points = rrect.to_points();
    let num = points.len() as isize;
    iter::from_fn(move || {
        let p = points[(index % num) as usize];
        index += step;
        Some(p)
    })
}

fn oval_point_iterator(
    rect: &Rect,
    dir_start: impl Into<Option<(Direction, usize)>>,
) -> impl Iterator<Item = Point> {
    let (dir, start) = dir_start.into().unwrap_or_default();

    let step = match dir {
        Direction::CW => 1,
        Direction::CCW => -1,
    };

    let mut index = start as isize;
    let rect = rect.clone();
    let center = rect.center();

    iter::from_fn(move || {
        let p = match index % 4 {
            0 => Point::new(center.left(), rect.top()),
            1 => Point::new(rect.right(), center.top()),
            2 => Point::new(center.left(), rect.bottom()),
            3 => Point::new(rect.left(), center.top()),
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
