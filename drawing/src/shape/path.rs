use crate::{
    point::point, scalar, Arc, Bounds, Circle, FastBounds, Matrix, Oval, Point, Radians, Rect,
    RoundedRect, Scalar, Vector,
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
    pub(crate) verbs: Vec<Verb>,
}

#[derive(Copy, Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum FillType {
    Winding,
    EvenOdd,
    InverseWinding,
    InverseEvenOdd,
}

impl FillType {
    pub fn is_inverse(&self) -> bool {
        use FillType::*;
        match self {
            Winding | EvenOdd => false,
            InverseWinding | InverseEvenOdd => true,
        }
    }
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

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub enum Verb {
    MoveTo(Point),
    LineTo(Point),
    QuadTo(Point, Point),
    ConicTo(Point, Point, scalar),
    CubicTo(Point, Point, Point),
    Close,
}

impl Verb {
    pub fn last_point(&self) -> Option<Point> {
        use Verb::*;
        match self {
            MoveTo(l) => Some(*l),
            LineTo(l) => Some(*l),
            QuadTo(_, l) => Some(*l),
            ConicTo(_, l, _) => Some(*l),
            CubicTo(_, _, l) => Some(*l),
            Close => None,
        }
    }
}

// TODO: add path combinators!

impl Path {
    pub fn is_empty(&self) -> bool {
        self.verbs.is_empty()
    }

    // TODO: return an iterator?
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
                Verb::Close => {}
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
    ) -> &mut Self {
        let (dir, start) = dir_start.into().unwrap_or_default();

        // CW: odd indices
        // CCW: even indices
        let starts_with_conic = ((start & 1) != 0) == (dir == Direction::CW);
        const WEIGHT: f64 = scalar::ROOT_2_OVER_2;

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
        self
    }

    pub fn add_oval(
        &mut self,
        oval: &Oval,
        dir_start: impl Into<Option<(Direction, usize)>>,
    ) -> &mut Self {
        self.reserve_verbs(6);
        const WEIGHT: f64 = std::f64::consts::FRAC_1_SQRT_2;

        let (dir, start) = dir_start.into().unwrap_or_default();

        let mut oval_iter = oval_point_iterator(oval.rect(), (dir, start));
        let mut rect_iter = rect_point_iterator(
            oval.rect(),
            (dir, start + if dir == Direction::CW { 0 } else { 1 }),
        );

        self.move_to(oval_iter.next().unwrap());
        for _ in 0..=3 {
            self.conic_to(rect_iter.next().unwrap(), oval_iter.next().unwrap(), WEIGHT);
        }
        self.close();
        self
    }

    pub fn add_circle(&mut self, circle: &Circle, dir: impl Into<Option<Direction>>) -> &mut Self {
        let dir = dir.into().unwrap_or_default();
        // TODO: does it make sense here to support a starting index?
        self.add_oval(&circle.to_oval(), (dir, 0))
    }

    pub fn add_rect(
        &mut self,
        rect: &Rect,
        dir_start: impl Into<Option<(Direction, usize)>>,
    ) -> &mut Self {
        let mut iter = rect_point_iterator(rect, dir_start);
        self.reserve_verbs(5)
            .move_to(iter.next().unwrap())
            .line_to(iter.next().unwrap())
            .line_to(iter.next().unwrap())
            .line_to(iter.next().unwrap())
            .close();
        self
    }

    pub fn add_polygon(&mut self, points: &[Point], close: bool) -> &mut Self {
        if points.is_empty() {
            return self;
        }

        self.reserve_verbs(points.len() + if close { 1 } else { 0 });

        self.move_to(points[0]);
        for point in &points[1..] {
            self.line_to(*point);
        }
        if close {
            self.close();
        }
        self
    }

    pub fn add_arc(&mut self, arc: &Arc) -> &Self {
        // Skia: 3a2e3e75232d225e6f5e7c3530458be63bbb355a

        /*
            note: Empty ovals and sweep_angle == 0 are fine for us.
            Assuming that his resolves to a single point, it may produce output
            depending on the Paint used.

            if (oval.is_empty() || sweep_angle == 0.0) {
                return self
            }
        */

        /* (armin) Also don't optimize for the Oval case.
        if sweep_angle >= Angle::FULL_CIRCLE || sweep_angle <= -Angle::FULL_CIRCLE {
            // We can treat the arc as an oval if it begins at one of our legal starting positions.
            // See SkPath::addOval() docs.
            SkScalar startOver90 = startAngle / 90.f;
            SkScalar startOver90I = SkScalarRoundToScalar(startOver90);
            SkScalar error = startOver90 - startOver90I;
            if (SkScalarNearlyEqual(error, 0)) {
                // Index 1 is at startAngle == 0.
                SkScalar startIndex = std::fmod(startOver90I + 1.f, 4.f);
                startIndex = startIndex < 0 ? startIndex + 4.f : startIndex;
                return this->addOval(oval, sweepAngle > 0 ? kCW_Direction : kCCW_Direction,
                (unsigned) startIndex);
            }
        }
        */

        self.arc_to(arc, true)
    }

    //
    // add primitive verbs.
    //

    #[allow(clippy::float_cmp)]
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
        let last_verb = self.verbs.last();
        if let Some(verb) = last_verb {
            match verb {
                Verb::MoveTo(_)
                | Verb::LineTo(_)
                | Verb::QuadTo(_, _)
                | Verb::ConicTo(_, _, _)
                | Verb::CubicTo(_, _, _) => {
                    self.add_verb(Verb::Close);
                }
                Verb::Close => {}
            }
        }
    }

    fn add_verb(&mut self, verb: Verb) -> &mut Self {
        self.verbs.push(verb);
        self
    }

    pub(crate) fn reserve_verbs(&mut self, additional: usize) -> &mut Self {
        self.verbs.reserve(additional);
        self
    }

    pub(crate) fn last_point(&self) -> Option<Point> {
        self.verbs.iter().rev().find_map(|v| v.last_point())
    }

    pub(crate) fn is_inverse_fill_type(&self) -> bool {
        self.fill_type.is_inverse()
    }

    pub(crate) fn bounds(&self) -> Option<Bounds> {
        Bounds::from_points(&self.points())
    }

    pub fn iter<'a>(&'a self, force_close: bool) -> impl Iterator<Item = PathVerb> + 'a {
        PathIterator::new(self, force_close)
    }
}

struct PathIterator<'a> {
    path: &'a Path,
    /// The last point moved to (and the first point for closing the contour).
    move_to: Point,
    last_pt: Point,
    force_close: bool,
    close_line: bool,
    segment_state: SegmentState,
    index: usize,
    need_close: bool,
}

impl PathIterator<'_> {
    fn new(path: &Path, force_close: bool) -> PathIterator {
        PathIterator {
            path,
            move_to: Point::default(),
            last_pt: Point::default(),
            force_close,
            close_line: false,
            segment_state: SegmentState::EmptyContour,
            index: 0,
            need_close: false,
        }
    }
}

pub enum PathVerb {
    MoveTo(Point),
    Line(Point, Point),
    Quad(Point, Point, Point),
    Conic(Point, Point, Point, scalar),
    Cubic(Point, Point, Point, Point),
    // TODO: try to remove that option.
    Close(Option<Point>),
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum SegmentState {
    EmptyContour,
    AfterMove,
    // Last point of the previous primitive.
    AfterPrimitive(Point),
}

impl SegmentState {
    pub fn is_after_primitive(&self) -> bool {
        match self {
            SegmentState::AfterPrimitive(_) => true,
            _ => false,
        }
    }
}

impl Iterator for PathIterator<'_> {
    type Item = PathVerb;
    fn next(&mut self) -> Option<Self::Item> {
        // Skia: 6d1c0d4196f19537cc64f74bacc7d123de3be454
        if self.index == self.path.verbs.len() {
            if self.need_close && self.segment_state.is_after_primitive() {
                let verb = self.auto_close();
                if let PathVerb::Line(_, _) = verb {
                    return Some(verb);
                }
                self.need_close = false;
                return Some(verb);
            }
            return None;
        }
        self.index += 1;
        match self.path.verbs[self.index - 1] {
            Verb::MoveTo(pt) => {
                if self.need_close {
                    self.index -= 1; // move back one verb
                    let verb = self.auto_close();
                    if let PathVerb::Close(_) = verb {
                        self.need_close = false;
                    }
                    return Some(verb);
                }
                if self.index == self.path.verbs.len() {
                    // might be a trailing moveto
                    return None;
                }
                self.move_to = pt;
                self.segment_state = SegmentState::AfterMove;
                self.last_pt = self.move_to;
                self.need_close = self.force_close;
                Some(PathVerb::MoveTo(pt))
            }
            Verb::LineTo(p1) => {
                let p0 = self.cons_move_to(p1);
                self.last_pt = p1;
                self.close_line = false;
                Some(PathVerb::Line(p0, p1))
            }
            Verb::QuadTo(p1, p2) => {
                let p0 = self.cons_move_to(p2);
                self.last_pt = p2;
                Some(PathVerb::Quad(p0, p1, p2))
            }
            Verb::ConicTo(p1, p2, w) => {
                let p0 = self.cons_move_to(p2);
                self.last_pt = p2;
                Some(PathVerb::Conic(p0, p1, p2, w))
            }
            Verb::CubicTo(p1, p2, p3) => {
                let p0 = self.cons_move_to(p3);
                self.last_pt = p3;
                Some(PathVerb::Cubic(p0, p1, p2, p3))
            }
            Verb::Close => {
                let verb = self.auto_close();
                if let PathVerb::Line(_, _) = verb {
                    self.index -= 1;
                } else {
                    self.need_close = false;
                    self.segment_state = SegmentState::EmptyContour
                }
                self.last_pt = self.move_to;
                Some(verb)
            }
        }
    }
}

impl PathIterator<'_> {
    fn auto_close(&mut self) -> PathVerb {
        // Skia: 6d1c0d4196f19537cc64f74bacc7d123de3be454
        if self.last_pt != self.move_to {
            // A special case: if both points are NaN, SkPoint::operation== returns
            // false, but the iterator expects that they are treated as the same.
            // (consider SkPoint is a 2-dimension float point).
            if self.last_pt.x.is_nan()
                || self.last_pt.y.is_nan()
                || self.move_to.x.is_nan()
                || self.move_to.y.is_nan()
            {
                return PathVerb::Close(None);
            }

            let line = PathVerb::Line(self.last_pt, self.move_to);
            self.last_pt = self.move_to;
            self.close_line = true;
            line
        } else {
            PathVerb::Close(Some(self.move_to))
        }
    }

    fn cons_move_to(&mut self, last_pt_prim: Point) -> Point {
        // Skia: 6d1c0d4196f19537cc64f74bacc7d123de3be454
        match self.segment_state {
            SegmentState::EmptyContour => panic!("unexpected segment state in cons_move_to()"),
            SegmentState::AfterMove => {
                self.segment_state = SegmentState::AfterPrimitive(last_pt_prim);
                self.move_to
            }
            // TODO: isn't this always last_pt?
            SegmentState::AfterPrimitive(point) => point,
        }
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
            0 => point(center.x, rect.top),
            1 => point(rect.right, center.y),
            2 => point(center.x, rect.bottom),
            3 => point(rect.left, center.y),
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
