//! port of some functions in core/SkGeometry.cpp
//! Skia: 6d1c0d4196f19537cc64f74bacc7d123de3be454

use super::scalar::{double_to_scalar, SCALAR_1};
use crate::path::Direction;
use crate::{scalar, Conic, Matrix, NearlyZero, Point, Scalar, Vector};

type S2 = Vector;
impl From<scalar> for S2 {
    fn from(s: scalar) -> Self {
        S2::new(s, s)
    }
}

fn from_point(p: Point) -> S2 {
    Vector::new(p.x, p.y)
}

fn to_point(v: S2) -> Point {
    v.into()
}

fn times_2(v: Vector) -> Vector {
    v * 2.0
}

// TODO: optimize using SSE
#[derive(Clone, PartialEq, Default, Debug)]
struct QuadCoeff {
    pub a: S2,
    pub b: S2,
    pub c: S2,
}

impl QuadCoeff {
    pub fn new(a: S2, b: S2, c: S2) -> Self {
        Self { a, b, c }
    }

    pub fn eval(&self, tt: impl Into<S2>) -> S2 {
        let tt = tt.into();
        (self.a * tt + self.b) * tt + self.c
    }
}

pub fn to_vector(s2: S2) -> Vector {
    s2
}

fn is_not_monotonic(a: scalar, b: scalar, c: scalar) -> bool {
    let ab = a - b;
    let mut bc = b - c;
    if ab < 0.0 {
        bc = -bc;
    }
    ab == 0.0 || bc < 0.0
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

// return_check_zero

fn return_check_zero(value: usize) -> usize {
    value
}

// SkFindUnitQuadRoots
pub fn find_unit_quad_roots(aa: scalar, bb: scalar, cc: scalar, roots: &mut [scalar; 2]) -> usize {
    if aa == 0.0 {
        return return_check_zero(valid_unit_divide(-cc, bb, &mut roots[0]));
    }

    let mut r = 0;

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
            roots.swap(0, 1);
        } else if roots[0] == roots[1] {
            // nearly-equal?
            r -= 1; // skip the double root
        }
    }
    return_check_zero(r)
}

// SkEvalQuadAt
// eval_quad_at

// SkEvalQuadTangentAt
// eval_quad_tangent_at

fn interp(v0: S2, v1: S2, t: S2) -> S2 {
    v0 + (v1 - v0) * t
}

// SkChopQuadAt
fn chop_quad_at(src: &[Point], dst: &mut [Point], t: scalar) {
    debug_assert!(src.len() == 3 && dst.len() == 5);
    debug_assert!(t > 0.0 && t < SCALAR_1);

    let p0 = from_point(src[0]);
    let p1 = from_point(src[1]);
    let p2 = from_point(src[2]);
    let tt = Vector::new(t, t);

    let p01 = interp(p0, p1, tt);
    let p12 = interp(p1, p2, tt);

    dst[0] = to_point(p0);
    dst[1] = to_point(p01);
    dst[2] = to_point(interp(p01, p12, tt));
    dst[3] = to_point(p12);
    dst[4] = to_point(p2);
}

// SkChopQuadAtHalf
// chop_quad_at_half

// SkFindQuadExtrema
// find_quad_extrema

fn flatten_double_quad_extrema_y(coords: &mut [Point]) {
    debug_assert!(coords.len() == 5);
    let y = coords[4 / 2].y;
    coords[6 / 2].y = y;
    coords[2 / 2].y = y;
}

/*  Returns 0 for 1 quad, and 1 for two quads, either way the answer is
stored in dst[]. Guarantees that the 1/2 quads will be monotonic.
*/
// SkChopQuadAtYExtrema
pub fn chop_quad_at_y_extrema(src: &[Point], dst: &mut [Point]) -> i32 {
    debug_assert!(src.len() == 3 && dst.len() == 5);

    let a = src[0].y;
    let mut b = src[1].y;
    let c = src[2].y;

    if is_not_monotonic(a, b, c) {
        let mut value: scalar = Default::default();
        if valid_unit_divide(a - b, a - b - b + c, &mut value) != 0 {
            chop_quad_at(src, dst, value);
            flatten_double_quad_extrema_y(dst);
            return 1;
        }
        // if we get here, we need to force dst to be monotonic, even though
        // we couldn't compute a unit_divide value (probably underflow).
        b = if (a - b).abs() < (b - c).abs() { a } else { c };
    }
    dst[0] = Point::new(src[0].x, a);
    dst[1] = Point::new(src[1].x, b);
    dst[2] = Point::new(src[2].x, c);
    return 0;
}

// SkChopQuadAtXExtrema
// chop_quad_at_x_extrema

// SkFindQuadMaxCurvature
// find_quad_max_curvature

// SkChopQuadAtMaxCurvature
// chop_quad_at_max_curvature

// SkConvertQuadToCubic
// convert_quad_to_cubic

fn eval_cubic_derivative(src: &[Point], t: scalar) -> Vector {
    debug_assert!(src.len() == 4);
    let mut coeff: QuadCoeff = Default::default();
    let p0 = from_point(src[0]);
    let p1 = from_point(src[1]);
    let p2 = from_point(src[2]);
    let p3 = from_point(src[3]);

    coeff.a = p3 + from_scalar(3.0) * (p1 - p2) - p0;
    coeff.b = times_2(p2 - times_2(p1) + p0);
    coeff.c = p1 - p0;
    to_vector(coeff.eval(t))
}

// eval_cubic_2ndDerivative
// eval_cubic_2nd_derivative

// SkEvalCubicAt
// eval_cubic_at_loc
// eval_cubic_at_curvature

pub fn eval_cubic_at_tangent(src: &[Point], t: scalar) -> Vector {
    debug_assert!(src.len() == 4);
    debug_assert!(t >= 0.0 && t <= SCALAR_1);
    let mut tangent: Vector;

    // The derivative equation returns a zero tangent vector when t is 0 or 1, and the
    // adjacent control point is equal to the end point. In this case, use the
    // next control point or the end points to compute the tangent.
    if (t == 0.0 && src[0] == src[1]) || (t == 1.0 && src[2] == src[3]) {
        if t == 0.0 {
            tangent = src[2] - src[0];
        } else {
            tangent = src[3] - src[1];
        }
        if tangent.x == 0.0 && tangent.y == 0.0 {
            tangent = src[3] - src[0];
        }
    } else {
        tangent = eval_cubic_derivative(src, t);
    }

    tangent
}

/** Cubic'(t) = At^2 + Bt + C, where
    A = 3(-a + 3(b - c) + d)
    B = 6(a - 2b + c)
    C = 3(b - a)
    Solve for t, keeping only those that fit between 0 < t < 1
*/
// SkFindCubicExtrema
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

// SkChopCubicAt
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
}

pub fn chop_cubic_at2(src: &[Point; 4], dst: &mut [Point; 10], values: &[scalar; 2], roots: usize) {
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
        let mut tmp: [Point; 4] = Default::default();

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

// SkChopCubicAtHalf
// chop_cubic_at_half

pub fn flatten_double_cubic_extrema_y(coords: &mut [Point]) {
    debug_assert!(coords.len() == 7);
    let y = coords[6 / 2].y;
    coords[8 / 2].y = y;
    coords[4 / 2].y = y;
}

/** Given 4 points on a cubic bezier, chop it into 1, 2, 3 beziers such that
    the resulting beziers are monotonic in Y. This is called by the scan
    converter.  Depending on what is returned, dst[] is treated as follows:
    0   dst[0..3] is the original cubic
    1   dst[0..3] and dst[3..6] are the two new cubics
    2   dst[0..3], dst[3..6], dst[6..9] are the three new cubics
    If dst == null, it is ignored and only the count is returned.
*/
// SkChopCubicAtYExtrema
pub fn chop_cubic_at_y_extrema(src: &[Point; 4], dst: &mut [Point; 10]) -> usize {
    let mut values: [scalar; 2] = Default::default();
    let roots = find_cubic_extrema(src[0].y, src[1].y, src[2].y, src[3].y, &mut values);

    chop_cubic_at2(src, dst, &values, roots);
    if roots > 0 {
        // we do some cleanup to ensure our Y extrema are flat
        flatten_double_cubic_extrema_y(&mut dst[0..7]);
        if roots == 2 {
            flatten_double_cubic_extrema_y(&mut dst[3..]);
        }
    }
    return roots;
}

// SkChopCubicAtXExtrema
// SkFindCubicInflections
// SkChopCubicAtInflections
// calc_dot_cross_cubic
// previous_inverse_pow2
// write_cubic_inflection_roots
// SkClassifyCubic
// bubble_sort
// collaps_duplicates
// SkScalarCubeRoot
// solve_cubic_poly
// formulate_F1DotF2
// SkFindCubicMaxCurvature
// SkChopCubicAtMaxCurvature
// calc_cubic_precision
// on_same_side
// SkFindCubicCusp
// cubic_dchop_at_intercept
// SkChopMonoCubicAtY
// SkChopMonoCubicAtX

fn conic_deriv_coeff_y(src: &[Point], w: scalar) -> [scalar; 3] {
    debug_assert!(src.len() == 3);

    let p20 = src[4 / 2].y - src[0 / 2].y;
    let p10 = src[2 / 2].y - src[0 / 2].y;
    let wp10 = w * p10;
    let c0 = w * p20 - p20;
    let c1 = p20 - 2.0 * wp10;
    let c2 = wp10;
    [c0, c1, c2]
}

fn conic_find_extrema_y(src: &[Point], w: scalar) -> Option<scalar> {
    debug_assert!(src.len() == 3);
    let coeff = conic_deriv_coeff_y(src, w);

    let mut values: [scalar; 2] = Default::default();
    let roots = find_unit_quad_roots(coeff[0], coeff[1], coeff[2], &mut values);
    debug_assert!(0 == roots || 1 == roots);

    if 1 == roots {
        Some(values[0])
    } else {
        None
    }
}

// p3d_interp
// ratquad_mapTo3D
// project_down

impl Conic {
    /*
    // SkConic::chopAt
    fn chop_at(&self, t: scalar, dst: &mut [Conic; 2]) -> bool {
        let tmp : [Point3; 3];
        let tmp2 : [Point3; 2];

        ratquad_mapTo3D(fPts, fW, tmp);

        p3d_interp(&tmp[0].fX, &tmp2[0].fX, t);
        p3d_interp(&tmp[0].fY, &tmp2[0].fY, t);
        p3d_interp(&tmp[0].fZ, &tmp2[0].fZ, t);

        dst[0].fPts[0] = fPts[0];
        dst[0].fPts[1] = project_down(tmp2[0]);
        dst[0].fPts[2] = project_down(tmp2[1]); dst[1].fPts[0] = dst[0].fPts[2];
        dst[1].fPts[1] = project_down(tmp2[2]);
        dst[1].fPts[2] = fPts[2];

        // to put in "standard form", where w0 and w2 are both 1, we compute the
        // new w1 as sqrt(w1*w1/w0*w2)
        // or
        // w1 /= sqrt(w0*w2)
        //
        // However, in our case, we know that for dst[0]:
        //     w0 == 1, and for dst[1], w2 == 1
        //
        SkScalar root = SkScalarSqrt(tmp2[1].fZ);
        dst[0].fW = tmp2[0].fZ / root;
        dst[1].fW = tmp2[2].fZ / root;
        SkASSERT(sizeof(dst[0]) == sizeof(SkScalar) * 7);
        SkASSERT(0 == offsetof(SkConic, fPts[0].fX));
        return SkScalarsAreFinite(&dst[0].fPts[0].fX, 7 * 2);
    }
    */

    // SkConic::evalAt
    // SkConic::evalTangentAt
    // SkConic::evalAt
}

// subdivide_w_value
// SkConic::chop
// SkConic::computeAsQuadError
// SkConic::asQuadTol
// kMaxConicToQuadPOW2
// SkConic::computeQuadPOW2
// between
// subdivide

impl Conic {
    // SkConic::chopIntoQuadsPOW2
    // SkConic::findXExtrema
    // SkConic::findYExtrema
    pub fn find_y_extrema(&self) -> Option<scalar> {
        conic_find_extrema_y(&self.points, self.weight)
    }

    // SkConic::chopAtXExtrema
    // SkConic::chopAtYExtrema
    /*
    pub fn chop_at_y_extrema(&self, dst: &mut [Conic; 2]) -> bool {
        if let Some(t) = self.find_y_extrema() {
            if (!self.chop_at(t, dst)) {
                // if chop can't return finite values, don't chop
                return false;
            }
            // now clean-up the middle, since we know t was meant to be at
            // an Y-extrema
            let value = dst[0].points[2].y;
            dst[0].points[1].y = value;
            dst[1].points[0].y = value;
            dst[1].points[1].y = value;
            return true;
        }
        return false;
    }
    */

    // SkConic::computeTightBounds
    // SkConic::computeFastBounds
    // SkConic::TransformW

    // This function is an older port compared to the rest of this file:
    // from Skia: 3a2e3e75232d225e6f5e7c3530458be63bbb355a
    // SkConic::BuildUnitArc
    pub fn build_unit_arc(
        u_start: &Vector,
        u_stop: &Vector,
        dir: Direction,
        user_matrix: Option<&Matrix>,
    ) -> Vec<Conic> {
        // rotate by x,y so that uStart is (1.0)
        let x = Vector::dot_product(u_start, u_stop);
        let mut y = Vector::cross_product(u_start, u_stop);

        let abs_y = y.abs();

        // check for (effectively) coincident vectors
        // this can happen if our angle is nearly 0 or nearly 180 (y == 0)
        // ... we use the dot-prod to distinguish between 0 and 180 (x > 0)
        if abs_y <= scalar::NEARLY_ZERO
            && x > 0.0
            && ((y >= 0.0 && Direction::CW == dir) || (y <= 0.0 && Direction::CCW == dir))
        {
            return Vec::new();
        }

        if dir == Direction::CCW {
            y = -y;
        }

        // We decide to use 1-conic per quadrant of a circle. What quadrant does [xy] lie in?
        //      0 == [0  .. 90)
        //      1 == [90 ..180)
        //      2 == [180..270)
        //      3 == [270..360)
        //
        let mut quadrant = 0;
        if y == 0.0 {
            quadrant = 2; // 180
            debug_assert!((x + 1.0).abs() <= scalar::NEARLY_ZERO);
        } else if x == 0.0 {
            debug_assert!(abs_y - 1.0 <= scalar::NEARLY_ZERO);
            quadrant = if y > 0.0 { 1 } else { 3 }; // 90 : 270
        } else {
            if y < 0.0 {
                quadrant += 2;
            }
            if (x < 0.0) != (y < 0.0) {
                quadrant += 1;
            }
        }

        const QUADRANT_PTS: [Point; 8] = [
            Point::new(1.0, 0.0),
            Point::new(1.0, 1.0),
            Point::new(0.0, 1.0),
            Point::new(-1.0, 1.0),
            Point::new(-1.0, 0.0),
            Point::new(-1.0, -1.0),
            Point::new(0.0, -1.0),
            Point::new(1.0, -1.0),
        ];
        const QUADRANT_WEIGHT: scalar = scalar::ROOT_2_OVER_2;

        let mut dst = vec![Conic::default(); quadrant];
        for i in 0..dst.len() {
            let q_i = i * 2;
            let mut points = [Point::default(); 3];
            points.copy_from_slice(&QUADRANT_PTS[q_i..q_i + 3]);
            dst[i] = Conic::from((points, QUADRANT_WEIGHT));
        }

        // Now compute any remaing (sub-90-degree) arc for the last conic
        let final_p = Vector::new(x, y);
        let last_q = QUADRANT_PTS[quadrant * 2].to_vector(); // will already be a unit-vector
        let dot = Vector::dot_product(&last_q, &final_p);
        debug_assert!(0.0 <= dot && dot <= 1.0 + scalar::NEARLY_ZERO);

        if dot < 1.0 {
            let mut off_curve = Vector::new(last_q.x + x, last_q.y + y);
            // compute the bisector vector, and then rescale to be the off-curve point.
            // we compute its length from cos(theta/2) = length / 1, using half-angle identity we get
            // length = sqrt(2 / (1 + cos(theta)). We already have cos() when to computed the dot.
            // This is nice, since our computed weight is cos(theta/2) as well!
            //
            let cos_theta_over2 = ((1.0 + dot) / 2.0).sqrt();
            off_curve.set_length(cos_theta_over2.invert());
            // note (armin): Skia's implementation calls out to SkPointPriv::EqualsWithinTolerance, which
            // has zero tolerance and just checks if the scalars are finite.
            if last_q != off_curve {
                dst.push(Conic::new(
                    &[last_q.into(), off_curve.into(), final_p.into()],
                    cos_theta_over2,
                ));
            }
        }

        // now handle counter-clockwise and the initial unitStart rotation
        let mut matrix = Matrix::new_sin_cos(u_start.y, u_start.x, None);
        if dir == Direction::CCW {
            matrix.pre_scale((1.0, -1.0), None);
        }

        if let Some(user_matrix) = user_matrix {
            matrix.post_concat(user_matrix);
        }
        for i in 0..dst.len() {
            matrix.map_points(&dst[i].points);
        }

        dst
    }
}

fn from_scalar(s: scalar) -> S2 {
    Vector::new(s, s)
}
