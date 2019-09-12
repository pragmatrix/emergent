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

        self
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

/*
pub(crate) mod tangent {
    // Skia: 6d1c0d4196f19537cc64f74bacc7d123de3be454
    use super::SCALAR_1;
    use crate::{scalar, NearlyEqual, NearlyZero, Point, Scalar, Vector};
    use std::mem;

    pub fn cubic(pts: &[Point; 4], x: scalar, y: scalar, tangents: &mut Vec<Vector>) {
        if !between(pts[0].y, y, pts[1].y)
            && !between(pts[1].y, y, pts[2].y)
            && !between(pts[2].y, y, pts[3].y)
        {
            return;
        }
        if !between(pts[0].x, x, pts[1].x)
            && !between(pts[1].x, x, pts[2].x)
            && !between(pts[2].x, x, pts[3].x)
        {
            return;
        }
        let mut dst: [Point; 10];
        let n = chop_cubic_at_y_extrema(pts, &mut dst);
        for i in 0..=n {
            let c = &dst[i * 3..i * 3 + 4];
            let mut t: scalar;
            if !super::cubic_clipper::chop_mono_at_y(c, y, &mut t) {
                continue;
            }
            let xt = eval_cubic_pts(c[0].x, c[1].x, c[2].x, c[3].x, t);
            if !scalar_nearly_equal(x, xt) {
                continue;
            }
            let tangent: Vector;
            eval_qubic_at(c, t, None, &tangent, None);
            tangents.push(tangent);
        }
    }

    /** Given 4 points on a cubic bezier, chop it into 1, 2, 3 beziers such that
        the resulting beziers are monotonic in Y. This is called by the scan
        converter.  Depending on what is returned, dst[] is treated as follows:
        0   dst[0..3] is the original cubic
        1   dst[0..3] and dst[3..6] are the two new cubics
        2   dst[0..3], dst[3..6], dst[6..9] are the three new cubics
        If dst == null, it is ignored and only the count is returned.
    */
    fn chop_cubic_at_y_extrema(src: &[Point; 4], dst: &mut [Point; 10]) -> usize {
        let mut values: [scalar; 2] = Default::default();
        let roots = find_cubic_extrema(src[0].y, src[1].y, src[2].y, src[3].y, &mut values);

        chop_cubic_at2(src, dst, values, roots);
        if (dst && roots > 0) {
            // we do some cleanup to ensure our Y extrema are flat
            flatten_double_cubic_extrema(&dst[0].y);
            if (roots == 2) {
                flatten_double_cubic_extrema(&dst[3].y);
            }
        }
        return roots;
    }

    /** Cubic'(t) = At^2 + Bt + C, where
        A = 3(-a + 3(b - c) + d)
        B = 6(a - 2b + c)
        C = 3(b - a)
        Solve for t, keeping only those that fit between 0 < t < 1
    */
    fn find_cubic_extrema(
        a: scalar,
        b: scalar,
        c: scalar,
        d: scalar,
        values: &mut [scalar; 2],
    ) -> usize {
        // we divide A,B,C by 3 to simplify
        let aa = d - a + 3.0 * (b - c);
        let bb = 2.0 * (a - b - b + c);
        let cc = b - a;

        find_unit_quad_roots(aa, bb, cc, values)
    }

    fn find_unit_quad_roots(aa: scalar, bb: scalar, cc: scalar, roots: &mut [scalar; 2]) -> usize {
        if aa == 0.0 {
            return return_check_zero(valid_unit_divide(-cc, bb, &mut roots[0]));
        }

        let r = 0;

        // use doubles so we don't overflow temporarily trying to compute R
        let mut dr = bb as f64 * bb - 4.0 * aa as f64 * cc;
        if dr < 0.0 {
            return return_check_zero(0);
        }
        dr = dr.sqrt();
        let rr = double_to_scalar(dr);
        if !rr.is_finite() {
            return return_check_zero(0);
        }

        let qq = if bb < 0.0 {
            -(bb - rr) / 2.0
        } else {
            -(bb + rr) / 2.0
        };
        r += valid_unit_divide(qq, aa, &mut roots[r]);
        r += valid_unit_divide(cc, qq, &mut roots[r]);
        if r == 2 {
            if roots[0] > roots[1] {
                mem::swap(&mut roots[0], &mut roots[1]);
            } else if roots[0] == roots[1] {
                // nearly-equal?
                r -= 1; // skip the double root
            }
        }
        return_check_zero(r)
    }

    fn valid_unit_divide(mut numer: scalar, mut denom: scalar, ratio: &mut scalar) -> usize {
        if numer < 0.0 {
            numer = -numer;
            denom = -denom;
        }

        if denom == 0.0 || numer == 0.0 || numer >= denom {
            return 0;
        }

        let r = numer / denom;
        if r.is_nan() {
            return 0;
        }
        debug_assert!(
            r >= 0.0 && r < SCALAR_1,
            "numer {}, denom {}, r {}",
            numer,
            denom,
            r
        );
        if r == 0.0 {
            // catch underflow if numer <<<< denom
            return 0;
        }
        *ratio = r;
        1
    }

    fn return_check_zero(value: usize) -> usize {
        value
    }

    fn chop_cubic_at(src: &[Point], dst: &mut [Point], t: scalar) {
        debug_assert!(src.len() == 3 && dst.len() == 7);
        debug_assert!(t > 0.0 && t < SCALAR_1);

        // TODO: may re-add SIMD support.

        let p0 = from_point(src[0]);
        let p1 = from_point(src[1]);
        let p2 = from_point(src[2]);
        let p3 = from_point(src[3]);
        let tt = Vector::new(t, t);

        let ab = interp(p0, p1, tt);
        let bc = interp(p1, p2, tt);
        let cd = interp(p2, p3, tt);
        let abc = interp(ab, bc, tt);
        let bcd = interp(bc, cd, tt);
        let abcd = interp(abc, bcd, tt);

        dst[0] = to_point(p0);
        dst[1] = to_point(ab);
        dst[2] = to_point(abc);
        dst[3] = to_point(abcd);
        dst[4] = to_point(bcd);
        dst[5] = to_point(cd);
        dst[6] = to_point(p3);

        fn from_point(p: Point) -> Vector {
            Vector::new(p.x, p.y)
        }

        fn to_point(v: Vector) -> Point {
            v.into()
        }

        fn interp(v0: Vector, v1: Vector, t: Vector) -> Vector {
            v0 + (v1 - v0) * t
        }
    }

    pub fn chop_cubic_at2(
        src: &[Point; 4],
        mut dst: &mut [Point; 10],
        values: &[scalar; 2],
        roots: usize,
    ) {
        #[cfg(debug_assertions)]
        {
            for i in 0..roots - 1 {
                debug_assert!(0.0 < values[i] && values[i] < 1.0);
                debug_assert!(0.0 < values[i + 1] && values[i + 1] < 1.0);
                debug_assert!(values[i] < values[i + 1]);
            }
        }

        if roots == 0 {
            // nothing to chop
            dst[0..4].copy_from_slice(src);
        } else {
            let mut t = values[0];
            let mut src = src.as_ref();
            let mut tmp: [Point; 4];

            let mut dst = dst.as_mut();
            for i in 0..roots {
                chop_cubic_at(src, &mut dst[0..6], t);
                if i == roots - 1 {
                    break;
                }

                dst = &mut dst[3..];
                // have src point to the remaining cubic (after the chop)
                tmp.copy_from_slice(&dst[0..4]);
                src = &tmp;

                // watch out in case the renormalized t isn't in range
                if valid_unit_divide(values[i + 1] - values[i], SCALAR_1 - values[i], &mut t) == 0 {
                    // if we can't, just create a degenerate cubic
                    let src3 = src[3];
                    dst[6] = src3;
                    dst[5] = src3;
                    dst[4] = src3;
                    break;
                }
            }
        }
    }

    pub fn line(pts: &[Point; 2], x: scalar, y: scalar, tangents: &mut Vec<Vector>) {
        let y0 = pts[0].y;
        let y1 = pts[1].y;
        if !between(y0, y, y1) {
            return;
        }
        let x0 = pts[0].x;
        let x1 = pts[1].x;
        if !between(x0, x, x1) {
            return;
        }
        let dx = x1 - x0;
        let dy = y1 - y0;
        if !scalar_nearly_equal((x - x0) * dy, dx * (y - y0)) {
            return;
        }
        let v = Vector::new(dx, dy);
        tangents.push(v);
    }

    fn between(a: scalar, b: scalar, c: scalar) -> bool {
        debug_assert!(
            ((a <= b && b <= c) || (a >= b && b >= c)) == ((a - b) * (c - b) <= 0.0)
                || (scalar_nearly_zero(a) && scalar_nearly_zero(b) && scalar_nearly_zero(c))
        );
        (a - b) * (c - b) <= 0.0
    }

    fn eval_cubic_pts(c0: scalar, c1: scalar, c2: scalar, c3: scalar, t: scalar) -> scalar {
        let aa = c3 + 3.0 * (c1 - c2) - c0;
        let bb = 3.0 * (c2 - c1 - c1 + c0);
        let cc = 3.0 * (c1 - c0);
        let dd = c0;
        poly_eval(aa, bb, cc, dd, t)
    }

    fn poly_eval(a: f64, b: f64, c: f64, d: f64, t: f64) -> f64 {
        ((a * t + b) * t + c) * t + d
    }

    fn scalar_nearly_zero(s: scalar) -> bool {
        s.nearly_zero(scalar::NEARLY_ZERO)
    }

    fn scalar_nearly_equal(a: scalar, b: scalar) -> bool {
        a.nearly_equal(&b, scalar::NEARLY_ZERO)
    }

    fn double_to_scalar(s: f64) -> scalar {
        s
    }
}

const SCALAR_1: scalar = 1.0;

mod geometry {
    use super::SCALAR_1;
    use crate::{scalar, Point, Vector};

    pub fn eval_cubic_at_tangent(src: &[Point; 4], t: scalar, tangent: &mut Vector) {
        debug_assert!(t >= 0.0 && t <= SCALAR_1);

        // The derivative equation returns a zero tangent vector when t is 0 or 1, and the
        // adjacent control point is equal to the end point. In this case, use the
        // next control point or the end points to compute the tangent.
        if ((t == 0.0 && src[0] == src[1]) || (t == 1.0 && src[2] == src[3])) {
            if (t == 0.0) {
                *tangent = src[2] - src[0];
            } else {
                *tangent = src[3] - src[1];
            }
            if (tangent.x == 0.0 && tangent.y == 0.0) {
                *tangent = src[3] - src[0];
            }
        } else {
            *tangent = eval_cubic_derivative(src, t);
        }
    }

    fn eval_cubic_derivative(src: &[Point; 4], t: scalar) -> Vector {
SkQuadCoeff coeff;
Sk2s P0 = from_point(src[0]);
Sk2s P1 = from_point(src[1]);
Sk2s P2 = from_point(src[2]);
Sk2s P3 = from_point(src[3]);

coeff.fA = P3 + Sk2s(3) * (P1 - P2) - P0;
coeff.fB = times_2(P2 - times_2(P1) + P0);
coeff.fC = P1 - P0;
return to_vector(coeff.eval(t));
}
}

mod cubic_clipper {
    use super::SCALAR_1;
    use crate::{scalar, Point};

    pub fn chop_mono_at_y(pts: &[Point], y: scalar, t: &mut scalar) -> bool {
        debug_assert!(pts.len() == 4);
        let mut ycrv = [pts[0].y - y, pts[1].y - y, pts[2].y - y, pts[3].y - y];

        // Check that the endpoints straddle zero.
        let t_neg: scalar; // Negative and positive function parameters.
        let t_pos: scalar;
        if ycrv[0] < 0.0 {
            if ycrv[3] < 0.0 {
                return false;
            }
            t_neg = 0.0;
            t_pos = SCALAR_1;
        } else if ycrv[0] > 0.0 {
            if ycrv[3] > 0.0 {
                return false;
            }
            t_neg = SCALAR_1;
            t_pos = 0.0;
        } else {
            *t = 0.0;
            return true;
        }

        let tol = SCALAR_1 / 65536.0; // 1 for fixed, 1e-5 for float.
        let iters = 0;
        loop {
            let t_mid = (t_pos + t_neg) / 2.0;
            let y01 = scalar_interp(ycrv[0], ycrv[1], t_mid);
            let y12 = scalar_interp(ycrv[1], ycrv[2], t_mid);
            let y23 = scalar_interp(ycrv[2], ycrv[3], t_mid);
            let y012 = scalar_interp(y01, y12, t_mid);
            let y123 = scalar_interp(y12, y23, t_mid);
            let y0123 = scalar_interp(y012, y123, t_mid);
            if y0123 == 0.0 {
                *t = t_mid;
                return true;
            }
            if y0123 < 0.0 {
                t_neg = t_mid;
            } else {
                t_pos = t_mid;
            }
            iters += 1;
            // Nan-safe
            if (t_pos - t_neg).abs() <= tol {
                break;
            }
        }

        *t = (t_neg + t_pos) / 2.0;
        return true;
    }

    fn scalar_interp(a: scalar, b: scalar, t: scalar) -> scalar {
        debug_assert!(t >= 0.0 && t <= SCALAR_1);
        a + (b - a) * t
    }
}
*/
