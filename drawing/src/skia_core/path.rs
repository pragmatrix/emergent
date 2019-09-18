// Skia: 6d1c0d4196f19537cc64f74bacc7d123de3be454
use crate::path::Direction;
use crate::skia_core::geometry;
use crate::skia_core::geometry::{chop_quad_at_y_extrema, find_unit_quad_roots};
use crate::skia_core::scalar::SignAsInt;
use crate::{
    scalar, Angle, Arc, Bounds, Conic, Matrix, NearlyEqual, NearlyZero, Path, Point, Radians, Rect,
    Vector,
};
use std::mem;

fn poly_eval_4(a: f64, b: f64, c: f64, t: f64) -> f64 {
    (a * t + b) * t + c
}

fn poly_eval(a: f64, b: f64, c: f64, d: f64, t: f64) -> f64 {
    ((a * t + b) * t + c) * t + d
}

// joinNoEmptyChecks
// is_degenerate
// SkPath::*

impl Path {
    pub fn arc_to(&mut self, arc: &Arc, force_move_to: bool) -> &Self {
        // Skia: 3a2e3e75232d225e6f5e7c3530458be63bbb355a
        let oval = arc.oval.rect();
        let start_angle = arc.start;
        let sweep_angle = arc.sweep;

        if oval.width() < 0.0 || oval.height() < 0.0 {
            return self;
        }

        let force_move_to = force_move_to || self.is_empty();

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
            let single_pt = Point::new(
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

    let start_v = Vector::new(
        start_rad.sin(), /*.snap_to_zero(NEARLY_ZERO)*/
        start_rad.cos(), /*.snap_to_zero(NEARLY_ZERO)*/
    );
    let stop_v = Vector::new(
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
    let mut matrix = Matrix::new_scale(Vector::new(oval.width() * 0.5, oval.height() * 0.5), None);
    matrix.post_translate(oval.center().to_vector());

    let conics = Conic::build_unit_arc(start, stop, dir, Some(&matrix));
    if conics.is_empty() {
        ArcConics::SinglePoint(matrix.map_point(*stop))
    } else {
        ArcConics::Conics(conics)
    }
}

fn contains_inclusive(r: &Rect, x: scalar, y: scalar) -> bool {
    return r.left <= x && x <= r.right && r.top <= y && y <= r.bottom;
}

fn contains_inclusive_bounds(bounds: Option<Bounds>, x: scalar, y: scalar) -> bool {
    match bounds {
        Some(b) => contains_inclusive(&b.to_rect(), x, y),
        None => false,
    }
}

/*
impl Contains<Point> for Path {
    fn contains(&self, p: Point) -> bool {
        let (x, y) = (p.x, p.y);
        let isInverse = self.is_inverse_fill_type();
        if self.is_empty() {
            return isInverse;
        }

        if !contains_inclusive_bounds(self.bounds(), x, y) {
        return isInverse;
        }

        let iter = self.iter(true);
        let done = false;
        let w = 0;
        let mut onCurveCount = 0;
        for verb in iter {
            use PathVerb::*;
            match verb {
                MoveTo(_) | Close(_) => {}
                Line(p0, p1) =>
                    w += winding_line(p0, p1, x, y, &mut onCurveCount),
                Quad(p0, p1, p2) =>
                    w += winding_quad(&[p0, p1, p2], x, y, &mut onCurveCount),
                Conic(p0, p1, p2, w) =>
                    w += winding_conic(pts, x, y, iter.conicWeight(), &onCurveCount),
                Cubic(p0, p1, p2, p3) =>
                    w += winding_cubic(pts, x, y, &onCurveCount),
            }
        };

        bool evenOddFill = SkPath::kEvenOdd_FillType == this->getFillType()
            || SkPath::kInverseEvenOdd_FillType == this->getFillType();
        if (evenOddFill) {
            w &= 1;
        }
        if (w) {
            return !isInverse;
        }
        if (onCurveCount <= 1) {
            return SkToBool(onCurveCount) ^ isInverse;
        }
        if ((onCurveCount & 1) || evenOddFill) {
            return SkToBool(onCurveCount & 1) ^ isInverse;
        }
        // If the point touches an even number of curves, and the fill is winding, check for
        // coincidence. Count coincidence as places where the on curve points have identical tangents.
        iter.setPath(*this, true);
        done = false;
        SkTDArray<SkVector> tangents;
        do {
            SkPoint pts[4];
            int oldCount = tangents.count();
            switch (iter.next(pts)) {
                case SkPath::kMove_Verb:
                    case SkPath::kClose_Verb:
                break;
                case SkPath::kLine_Verb:
                    tangent_line(pts, x, y, &tangents);
                break;
                case SkPath::kQuad_Verb:
                    tangent_quad(pts, x, y, &tangents);
                break;
                case SkPath::kConic_Verb:
                    tangent_conic(pts, x, y, iter.conicWeight(), &tangents);
                break;
                case SkPath::kCubic_Verb:
                    tangent_cubic(pts, x, y, &tangents);
                break;
                case SkPath::kDone_Verb:
                    done = true;
                break;
            }
            if (tangents.count() > oldCount) {
                int last = tangents.count() - 1;
                const SkVector& tangent = tangents[last];
                if (SkScalarNearlyZero(SkPointPriv::LengthSqd(tangent))) {
                    tangents.remove(last);
                } else {
                    for (int index = 0; index < last; ++index) {
                        const SkVector& test = tangents[index];
                        if (SkScalarNearlyZero(test.cross(tangent))
                            && SkScalarSignAsInt(tangent.fX * test.fX) <= 0
                            && SkScalarSignAsInt(tangent.fY * test.fY) <= 0) {
                            tangents.remove(last);
                            tangents.removeShuffle(index);
                            break;
                        }
                    }
                }
            }
        } while (!done);
        return SkToBool(tangents.count()) ^ isInverse;
    }
}
*/

// SkPath::ConvertConicToQuads
// SkPathPriv::IsSimpleClosedRect
// SkPathPriv::DrawArcIsConvex
// SkPathPriv::CreateDrawArcPath
// compute_quad_extremas
// compute_conic_extremas
// compute_cubic_extremas
// SkPath::computeTightBounds
// SkPath::IsLineDegenerate
// SkPath::IsQuadDegenerate
// SkPath::IsCubicDegenerate

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

// eval_cubic_pts
// find_minmax

// checkOnCurve
fn check_on_curve(x: scalar, y: scalar, start: Point, end: Point) -> bool {
    if start.y == end.y {
        return between(start.x, x, end.x) && x != end.x;
    } else {
        return x == start.x && y == start.y;
    }
}

// winding_mono_cubic
// winding_cubic
// conic_eval_numerator
// conic_eval_denominator
// winding_mono_conic

fn is_mono_quad(y0: scalar, y1: scalar, y2: scalar) -> bool {
    //    return SkScalarSignAsInt(y0 - y1) + SkScalarSignAsInt(y1 - y2) != 0;
    if y0 == y1 {
        return true;
    }
    if y0 < y1 {
        return y1 <= y2;
    } else {
        return y1 >= y2;
    }
}

/*
fn winding_conic(pts: &[Point], x: scalar, y: scalar, weight: scalar, on_curve_count: &mut i32) -> i32 {
    debug_assert!(pts.len() == 3);

    let conic = Conic::new(&[pts[0], pts[1], pts[2]], weight);
    let mut chopped : [Conic; 2] = Default::default();
    // If the data points are very large, the conic may not be monotonic but may also
    // fail to chop. Then, the chopper does not split the original conic in two.
    let isMono = is_mono_quad(pts[0].y, pts[1].y, pts[2].y) || !conic.chopAtYExtrema(chopped);
    int w = winding_mono_conic(isMono ? conic : chopped[0], x, y, onCurveCount);
    if (!isMono) {
        w += winding_mono_conic(chopped[1], x, y, onCurveCount);
    }
    return w;
}
*/

fn winding_mono_quad(pts: &[Point], x: scalar, y: scalar, on_curve_count: &mut i32) -> i32 {
    debug_assert!(pts.len() == 3);
    let mut y0 = pts[0].y;
    let mut y2 = pts[2].y;

    let mut dir: i32 = 1;
    if y0 > y2 {
        mem::swap(&mut y0, &mut y2);
        dir = -1;
    }
    if y < y0 || y > y2 {
        return 0;
    }
    if check_on_curve(x, y, pts[0], pts[2]) {
        *on_curve_count += 1;
        return 0;
    }
    if y == y2 {
        return 0;
    }
    // bounds check on X (not required. is it faster?)
    /*
    if (pts[0].fX > x && pts[1].fX > x && pts[2].fX > x) {
        return 0;
    }
    */

    let mut roots: [scalar; 2] = Default::default();
    let n = find_unit_quad_roots(
        pts[0].y - 2.0 * pts[1].y + pts[2].y,
        2.0 * (pts[1].y - pts[0].y),
        pts[0].y - y,
        &mut roots,
    );
    debug_assert!(n <= 1);
    let xt: scalar;
    if 0 == n {
        // zero roots are returned only when y0 == y
        // Need [0] if dir == 1
        // and  [2] if dir == -1
        xt = pts[(1 - dir) as usize].x;
    } else {
        let t = roots[0];
        let cc = pts[0].x;
        let aa = pts[2].x - 2.0 * pts[1].x + cc;
        let bb = 2.0 * (pts[1].x - cc);
        xt = poly_eval_4(aa, bb, cc, t);
    }
    if scalar_nearly_equal(xt, x) {
        if x != pts[2].x || y != pts[2].y {
            // don't test end points; they're start points
            *on_curve_count += 1;
            return 0;
        }
    }
    return if xt < x { dir } else { 0 };
}

fn winding_quad(pts: &[Point], x: scalar, y: scalar, on_curve_count: &mut i32) -> i32 {
    debug_assert!(pts.len() == 3);
    let mut dst: [Point; 5] = Default::default();
    let mut n = 0;

    let pts = {
        if !is_mono_quad(pts[0].y, pts[1].y, pts[2].y) {
            n = chop_quad_at_y_extrema(pts, &mut dst);
            &mut dst
        } else {
            pts
        }
    };
    let mut w = winding_mono_quad(&pts[0..3], x, y, on_curve_count);
    if n > 0 {
        w += winding_mono_quad(&pts[2..], x, y, on_curve_count);
    }
    w
}

// winding_line

fn winding_line(p0: Point, p1: Point, x: scalar, y: scalar, on_curve_count: &mut i32) -> i32 {
    let (x0, mut y0) = (p0.x, p0.y);
    let (x1, mut y1) = (p1.x, p1.y);

    let dy = y1 - y0;

    let mut dir = 1;
    if y0 > y1 {
        mem::swap(&mut y0, &mut y1);
        dir = -1;
    }
    if y < y0 || y > y1 {
        return 0;
    }
    if check_on_curve(x, y, p0, p1) {
        *on_curve_count += 1;
        return 0;
    }
    if y == y1 {
        return 0;
    }
    let cross = (x1 - x0) * (y - p0.y) - dy * (x - x0);

    if cross == 0.0 {
        // zero cross means the point is on the line, and since the case where
        // y of the query point is at the end point is handled above, we can be
        // sure that we're on the line (excluding the end point) here
        if x != x1 || y != p1.y {
            *on_curve_count += 1;
        }
        dir = 0;
    } else if cross.sign_as_int() == dir {
        dir = 0;
    }
    return dir;
}

pub fn tangent_cubic(pts: &[Point; 4], x: scalar, y: scalar, tangents: &mut Vec<Vector>) {
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
    let mut dst: [Point; 10] = Default::default();
    let n = geometry::chop_cubic_at_y_extrema(pts, &mut dst);
    for i in 0..=n {
        let c = &dst[i * 3..i * 3 + 4];
        let mut t: scalar = Default::default();
        if !super::cubic_clipper::chop_mono_at_y(c, y, &mut t) {
            continue;
        }
        let xt = eval_cubic_pts(c[0].x, c[1].x, c[2].x, c[3].x, t);
        if !scalar_nearly_equal(x, xt) {
            continue;
        }
        let tangent = geometry::eval_cubic_at_tangent(c, t);
        tangents.push(tangent);
    }
}

// tangent_conic
// tangent_quad

pub fn tangent_line(pts: &[Point; 2], x: scalar, y: scalar, tangents: &mut Vec<Vector>) {
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

fn scalar_nearly_zero(s: scalar) -> bool {
    s.nearly_zero(scalar::NEARLY_ZERO)
}

fn scalar_nearly_equal(a: scalar, b: scalar) -> bool {
    a.nearly_equal(&b, scalar::NEARLY_ZERO)
}
