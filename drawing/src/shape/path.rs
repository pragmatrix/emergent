use crate::{
    point::point, scalar, vector::vector, Angle, Arc, Bounds, Circle, Conic, FastBounds, Matrix,
    NearlyEqual, NearlyZero, Oval, Point, Radians, Rect, RoundedRect, Scalar, Vector,
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
        match self {
            Verb::MoveTo(l) => Some(*l),
            Verb::LineTo(l) => Some(*l),
            Verb::QuadTo(_, l) => Some(*l),
            Verb::ConicTo(_, l, _) => Some(*l),
            Verb::CubicTo(_, _, l) => Some(*l),
            Verb::Close => None,
        }
    }
}

// TODO: add path combinators!

impl Path {
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

    pub fn arc_to(&mut self, arc: &Arc, force_move_to: bool) -> &Self {
        // Skia: 3a2e3e75232d225e6f5e7c3530458be63bbb355a
        let oval = arc.oval.rect();
        let start_angle = arc.start;
        let sweep_angle = arc.sweep;

        if oval.width() < 0.0 || oval.height() < 0.0 {
            return self;
        }

        let force_move_to = force_move_to || self.verbs.is_empty();

        if let Some(lone_pt) = arc_is_lone_point(oval, start_angle, sweep_angle) {
            return if force_move_to {
                self.move_to(lone_pt)
            } else {
                self.line_to(lone_pt)
            };
        }

        let (start_v, stop_v, dir) = angles_to_unit_vectors(start_angle, sweep_angle);
        let add_pt = |path: &mut Path, pt: Point| {
            if force_move_to {
                path.move_to(pt);
                return;
            }

            match path.last_point() {
                Some(last_pt) if !last_pt.nearly_equal(&pt, scalar::NEARLY_ZERO) => {
                    path.line_to(pt);
                }
                None => {
                    path.line_to(pt);
                }
                _ => {}
            }
        };

        if start_v == stop_v {
            let end_angle: Radians = (start_angle + sweep_angle).into();
            let (radius_x, radius_y) = (oval.width() / 2.0, oval.height() / 2.0);
            let single_pt = point(
                oval.center().x + radius_x * (*end_angle).cos(),
                oval.center().y + radius_y * (*end_angle).sin(),
            );
            add_pt(self, single_pt);
            return self;
        }

        match build_arc_conics(oval, &start_v, &stop_v, dir) {
            ArcConics::Conics(conics) => {
                self.reserve_verbs(conics.len() * 2 + 1);
                add_pt(self, conics[0].points[0]);
                for conic in conics {
                    self.conic_to(conic.points[1], conic.points[2], conic.weight);
                }
            }
            ArcConics::SinglePoint(point) => add_pt(self, point),
        }

        return self;
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

    fn reserve_verbs(&mut self, additional: usize) -> &mut Self {
        self.verbs.reserve(additional);
        self
    }

    fn last_point(&self) -> Option<Point> {
        self.verbs.iter().rev().find_map(|v| v.last_point())
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

fn arc_is_lone_point(oval: &Rect, start_angle: Angle, sweep_angle: Angle) -> Option<Point> {
    // Skia: 3a2e3e75232d225e6f5e7c3530458be63bbb355a
    if sweep_angle == Angle::ZERO
        && (start_angle == Angle::ZERO || start_angle == Angle::FULL_CIRCLE)
    {
        // TODO: why right/centery ?
        return Some(Point::new(oval.right, oval.center().y));
    }
    if oval.width() == 0.0 && oval.height() == 0.0 {
        // TODO: why right / top
        return Some(oval.right_top());
    }

    None
}

// Note: implementation differs from the Skia version:
// - no snap to zero.
// - no adjustments for coincent vectors.

fn angles_to_unit_vectors(start_angle: Angle, sweep_angle: Angle) -> (Vector, Vector, Direction) {
    // Skia: 3a2e3e75232d225e6f5e7c3530458be63bbb355a
    let start_rad: scalar = start_angle.to_radians();
    let stop_rad: scalar = (start_angle + sweep_angle).to_radians();

    let start_v = vector(
        start_rad.sin(), /*.snap_to_zero(NEARLY_ZERO)*/
        start_rad.cos(), /*.snap_to_zero(NEARLY_ZERO)*/
    );
    let stop_v = vector(
        stop_rad.sin(), /*.snap_to_zero(NEARLY_ZERO)*/
        stop_rad.cos(), /*.snap_to_zero(NEARLY_ZERO)*/
    );

    /*
    If the sweep angle is nearly (but less than) 360, then due to precision
     loss in radians-conversion and/or sin/cos, we may end up with coincident
     vectors, which will fool SkBuildQuadArc into doing nothing (bad) instead
     of drawing a nearly complete circle (good).
     e.g. canvas.drawArc(0, 359.99, ...)
     -vs- canvas.drawArc(0, 359.9, ...)
     We try to detect this edge case, and tweak the stop vector

     */

    // TODO: I am not sure if this is needed anymore (armin).
    // TODO: needs testcase

    /*
    let mut stopRad = stopRad;
    let mut stopV = stopV;

    if startV == stopV {
        let sw = (*sweepAngle).abs();
        if sw < 360.0 && sw > 359.0 {
            // make a guess at a tiny angle (in radians) to tweak by
            let deltaRad = (1.0 as f64 / 512.0).copysign(*sweepAngle);
            // not sure how much will be enough, so we use a loop
            while {
                stopRad -= deltaRad;
                stopV = vector(
                    stopRad.sin().snap_to_zero(NEARLY_ZERO),
                    stopRad.cos().snap_to_zero(NEARLY_ZERO),
                );
                startV == stopV
            } {}
        }
    }
    */

    let dir = if sweep_angle > Angle::ZERO {
        Direction::CW
    } else {
        Direction::CCW
    };
    (start_v, stop_v, dir)
}

enum ArcConics {
    SinglePoint(Point),
    Conics(Vec<Conic>),
}

fn build_arc_conics(oval: &Rect, start: &Vector, stop: &Vector, dir: Direction) -> ArcConics {
    // Skia: 3a2e3e75232d225e6f5e7c3530458be63bbb355a
    let mut matrix = Matrix::new_scale(vector(oval.width() * 0.5, oval.height() * 0.5), None);
    matrix.post_translate(oval.center().to_vector());

    let conics = Conic::build_unit_arc(start, stop, dir, Some(&matrix));
    if conics.is_empty() {
        ArcConics::SinglePoint(matrix.map_point(*stop))
    } else {
        ArcConics::Conics(conics)
    }
}

impl FastBounds for Path {
    fn fast_bounds(&self) -> Bounds {
        Bounds::from_points(&self.points()).unwrap()
    }
}
