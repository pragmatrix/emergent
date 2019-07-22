use crate::{
    scalar, Bounds, FastBounds, NearlyEqual, NearlyZero, Point, Radians, Radius, Rect, Vector,
};
use serde::{Deserialize, Serialize};
use std::{mem, slice};

#[derive(Clone, Serialize, Deserialize, PartialEq, Debug)]
pub struct Matrix([scalar; 9]);

impl Default for Matrix {
    fn default() -> Self {
        Matrix::new_identity()
    }
}

impl Matrix {
    pub const IDENTITY: Matrix = Self::new_identity();

    pub const fn new_identity() -> Self {
        Self([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0])
    }

    pub fn new_scale(s: impl Into<Vector>, p: impl Into<Option<Point>>) -> Self {
        let s = s.into();
        let p = p.into().unwrap_or_default();
        let sx = s.x();
        let sy = s.y();
        Self::new_scale_translate(sx, sy, p.x - sx * p.x, p.y - sy * p.y)
    }

    pub fn new_translate(d: impl Into<Vector>) -> Matrix {
        let d = d.into();
        Self::new_scale_translate(1.0, 1.0, d.x(), d.y())
    }

    pub fn new_rotate(radians: impl Into<Radians>, p: impl Into<Option<Point>>) -> Self {
        let radians = *radians.into();
        Self::new_sin_cos(radians.sin(), radians.cos(), p.into().unwrap_or_default())
    }

    pub(crate) fn new_sin_cos(sin: scalar, cos: scalar, p: impl Into<Option<Point>>) -> Self {
        let p = p.into().unwrap_or_default();
        let one_minus_cos = 1.0 - cos;
        Self([
            cos,
            -sin,
            sdot(sin, p.y, one_minus_cos, p.x),
            sin,
            cos,
            sdot(-sin, p.x, one_minus_cos, p.y),
            0.0,
            0.0,
            1.0,
        ])
    }

    fn new_scale_translate(sx: scalar, sy: scalar, tx: scalar, ty: scalar) -> Self {
        Self([sx, 0.0, tx, 0.0, sy, ty, 0.0, 0.0, 1.0])
    }

    fn new_skew(sx: scalar, sy: scalar, p: impl Into<Option<Point>>) -> Self {
        let p = p.into().unwrap_or_default();
        Self([1.0, sx, -sx * p.y, sy, 1.0, -sy * p.x, 0.0, 0.0, 1.0])
    }

    pub fn is_identity(&self) -> bool {
        self.type_mask().is_identity()
    }

    /// Returns true if Matrix is identity, or translates.
    pub fn is_translate(&self) -> bool {
        (self.type_mask() & !(TypeMask::TRANSLATE)).is_empty()
    }

    /// Returns true if SkMatrix at most scales and translates.
    pub fn is_scale_translate(&self) -> bool {
        self.type_mask().is_scale_translate()
    }

    pub fn has_perspective(&self) -> bool {
        self.persp_0() != 0.0 || self.persp_1() != 0.0 || self.persp_2() != 1.0
    }

    /// Presumably cached type mask (not implemented yet, always calls compute_type_mask())
    fn type_mask(&self) -> TypeMask {
        self.compute_type_mask()
    }

    /// Computation of the type mask. Different to the Skia implementation in that it does not
    /// compute RectStaysRect.
    fn compute_type_mask(&self) -> TypeMask {
        if self.persp_0() != 0.0 || self.persp_1() != 0.0 || self.persp_2() != 1.0 {
            return TypeMask::all();
        }

        let mut mask = TypeMask::empty();
        if self.trans_x() != 0.0 || self.trans_y() != 0.0 {
            mask |= TypeMask::TRANSLATE
        }

        let m00 = self.scale_x();
        let m01 = self.skew_x();
        let m10 = self.skew_y();
        let m11 = self.scale_y();

        if m01 != 0.0 || m10 != 0.0 {
            mask |= TypeMask::AFFINE | TypeMask::SCALE;
        } else {
            if m00 != 1.0 || m11 != 1.0 {
                mask |= TypeMask::SCALE
            }
        }

        mask
    }

    pub fn pre_translate(&mut self, d: Vector) {
        if self.is_translate() {
            self.0[TRANS_X] += d.x();
            self.0[TRANS_Y] += d.y();
            return;
        }
        self.pre_concat(&Self::new_translate(d));
    }

    pub fn post_translate(&mut self, d: Vector) {
        if self.has_perspective() {
            self.post_concat(&Self::new_translate(d));
        } else {
            self.0[TRANS_X] += d.x();
            self.0[TRANS_Y] += d.y();
        }
    }

    pub fn pre_scale(&mut self, s: impl Into<Vector>, p: impl Into<Option<Point>>) {
        let s = s.into();
        if s != Vector::new(1.0, 1.0) {
            return;
        }
        self.pre_concat(&Self::new_scale(s, p))
    }

    pub fn post_scale(&mut self, s: impl Into<Vector>, p: impl Into<Option<Point>>) {
        let s = s.into();
        if s != Vector::new(1.0, 1.0) {
            self.post_concat(&Self::new_scale(s, p))
        }
    }

    pub fn pre_rotate(&mut self, radians: impl Into<Radians>, p: impl Into<Option<Point>>) {
        self.pre_concat(&Self::new_rotate(radians, p))
    }

    pub fn post_rotate(&mut self, radians: impl Into<Radians>, p: impl Into<Option<Point>>) {
        self.post_concat(&Self::new_rotate(radians, p))
    }

    pub fn pre_skew(&mut self, sx: scalar, sy: scalar, p: impl Into<Option<Point>>) {
        self.pre_concat(&Self::new_skew(sx, sy, p))
    }

    pub fn post_skew(&mut self, sx: scalar, sy: scalar, p: impl Into<Option<Point>>) {
        self.post_concat(&Self::new_skew(sx, sy, p))
    }

    pub fn pre_concat(&mut self, mat: &Matrix) {
        *self = Self::concat(self, mat)
    }

    pub fn post_concat(&mut self, mat: &Matrix) {
        *self = Self::concat(mat, self);
    }

    pub fn map_point(&self, point: impl Into<Point>) -> Point {
        let mut r = point.into();
        self.map_points_inplace(slice::from_mut(&mut r));
        r
    }

    pub fn map_points(&self, points: &[Point]) -> Vec<Point> {
        let mut r = points.to_vec();
        self.map_points_inplace(r.as_mut_slice());
        r
    }

    pub fn map_points_inplace(&self, points: &mut [Point]) {
        MAP_POINTS_PROC[self.type_mask().bits() as usize](self, points);
    }

    pub fn map_vector(&mut self, vector: Vector) -> Vector {
        let mut r = vector;
        self.map_vectors_inplace(slice::from_mut(&mut r));
        r
    }

    pub fn map_vectors(&mut self, vectors: &[Vector]) -> Vec<Vector> {
        let mut r = vectors.to_vec();
        self.map_vectors_inplace(r.as_mut_slice());
        r
    }

    pub fn map_vectors_inplace(&self, vectors: &mut [Vector]) {
        debug_assert_eq!(mem::size_of::<Vector>(), mem::size_of::<Point>());
        if self.has_perspective() {
            let origin = self.map_point(Point::ZERO) - Point::ZERO;
            self.map_points_inplace(unsafe { mem::transmute_copy(&vectors) });
            vectors.iter_mut().for_each(|v| *v -= origin);
        } else {
            let mut tmp = self.clone();
            tmp.0[TRANS_X] = 0.0;
            tmp.0[TRANS_Y] = 0.0;
            tmp.map_points_inplace(unsafe { mem::transmute(vectors) })
        }
    }

    pub fn map_radius(&self, radius: Radius) -> Radius {
        let mut v = [Vector::new(*radius, 0.0), Vector::new(0.0, *radius)];
        self.map_vectors_inplace(&mut v);
        let d0 = v[0].length();
        let d1 = v[1].length();
        Radius::from((d0 * d1).sqrt())
    }

    /// Treats the bounds as a rectangle, applies the matrix to it and returns
    /// the new bounds of the resulting transformed rectangle.
    pub fn map_bounds(&self, bounds: Bounds) -> Bounds {
        let tm = self.type_mask();
        if tm.is_translate() {
            return bounds + self.trans();
        }
        if tm.is_scale_translate() {
            let s = Vector::from(self.scale());
            let t = self.trans();
            let r = Rect::from(bounds);
            // TODO: Rect::fast_bounds() shouldn't probably be used here.
            return Rect::from_points(r.left_top() * s + t, r.right_bottom() * s + t).fast_bounds();
        }
        let mut quad = bounds.to_quad();
        self.map_points_inplace(&mut quad);
        Bounds::from_points(&quad).unwrap()
    }

    fn trans(&self) -> Vector {
        Vector::new(self.trans_x(), self.trans_y())
    }

    fn scale(&self) -> (scalar, scalar) {
        (self.scale_x(), self.scale_y())
    }

    fn _skew(&self) -> (scalar, scalar) {
        (self.skew_x(), self.skew_y())
    }

    fn scale_x(&self) -> scalar {
        self.0[SCALE_X]
    }

    fn skew_x(&self) -> scalar {
        self.0[SKEW_X]
    }

    fn trans_x(&self) -> scalar {
        self.0[TRANS_X]
    }

    fn skew_y(&self) -> scalar {
        self.0[SKEW_Y]
    }

    fn scale_y(&self) -> scalar {
        self.0[SCALE_Y]
    }

    fn trans_y(&self) -> scalar {
        self.0[TRANS_Y]
    }

    fn persp_0(&self) -> scalar {
        self.0[PERSP_0]
    }

    fn persp_1(&self) -> scalar {
        self.0[PERSP_1]
    }

    fn persp_2(&self) -> scalar {
        self.0[PERSP_2]
    }

    pub fn concat(a: &Matrix, b: &Matrix) -> Matrix {
        let atm = a.type_mask();

        if atm.is_identity() {
            return b.clone();
        }

        let btm = b.type_mask();

        if btm.is_identity() {
            return a.clone();
        }

        if atm.is_scale_translate() && btm.is_scale_translate() {
            return Matrix::new_scale_translate(
                a.scale_x() * b.scale_x(),
                a.scale_y() * b.scale_y(),
                a.scale_x() * b.trans_x() + a.trans_x(),
                a.scale_y() * b.trans_y() + a.trans_y(),
            );
        }

        if atm.has_perspective() || btm.has_perspective() {
            return Matrix([
                rowcol3(&a.0, &b.0, 0, 0),
                rowcol3(&a.0, &b.0, 0, 1),
                rowcol3(&a.0, &b.0, 0, 2),
                rowcol3(&a.0, &b.0, 3, 0),
                rowcol3(&a.0, &b.0, 3, 1),
                rowcol3(&a.0, &b.0, 3, 2),
                rowcol3(&a.0, &b.0, 6, 0),
                rowcol3(&a.0, &b.0, 6, 1),
                rowcol3(&a.0, &b.0, 6, 2),
            ]);
        }

        Matrix([
            sdot(a.scale_x(), b.scale_x(), a.skew_x(), b.skew_y()),
            sdot(a.scale_x(), b.skew_x(), a.skew_x(), b.scale_y()),
            sdot(
                a.scale_x(),
                b.trans_x(),
                a.skew_x(),
                b.trans_y() + a.trans_x(),
            ),
            sdot(a.skew_y(), b.scale_x(), a.scale_y(), b.skew_y()),
            sdot(a.skew_y(), b.skew_x(), a.scale_y(), b.scale_y()),
            sdot(
                a.skew_y(),
                b.trans_x(),
                a.scale_y(),
                b.trans_y() + a.trans_y(),
            ),
            0.0,
            0.0,
            1.0,
        ])
    }
}

bitflags! {
    struct TypeMask : u8 {
        const TRANSLATE = 0b0001;
        const SCALE = 0b0010;
        const AFFINE = 0b0100;
        const PERSPECTIVE = 0b1000;
    }
}

impl TypeMask {
    const IDENTITY: TypeMask = TypeMask::empty();

    fn is_identity(self) -> bool {
        self == Self::IDENTITY
    }

    /// Returns trie if the type mask indicates that the matrix at most translates.
    fn is_translate(self) -> bool {
        (self & !TypeMask::TRANSLATE).is_empty()
    }

    fn is_scale_translate(self) -> bool {
        (self & !(TypeMask::TRANSLATE | TypeMask::SCALE)).is_empty()
    }

    fn has_perspective(self) -> bool {
        !(self & TypeMask::PERSPECTIVE).is_empty()
    }
}

const SCALE_X: usize = 0;
const SKEW_X: usize = 1;
const TRANS_X: usize = 2;
const SKEW_Y: usize = 3;
const SCALE_Y: usize = 4;
const TRANS_Y: usize = 5;
const PERSP_0: usize = 6;
const PERSP_1: usize = 7;
const PERSP_2: usize = 8;

fn sdot(a: scalar, b: scalar, c: scalar, d: scalar) -> scalar {
    a * b + c * d
}

fn rowcol3(row: &[scalar], col: &[scalar], row_i: usize, col_i: usize) -> scalar {
    row[row_i] * col[col_i] + row[row_i + 1] * col[col_i + 3] + row[row_i + 2] * col[col_i + 6]
}

const MAP_POINTS_PROC: [fn(&Matrix, &mut [Point]); 16] = [
    identity_points,
    trans_points,
    scale_points,
    scale_points,
    affine_points,
    affine_points,
    affine_points,
    affine_points,
    persp_points,
    persp_points,
    persp_points,
    persp_points,
    persp_points,
    persp_points,
    persp_points,
    persp_points,
];

fn identity_points(m: &Matrix, _: &mut [Point]) {
    debug_assert!(m.is_identity());
}

fn trans_points(m: &Matrix, points: &mut [Point]) {
    debug_assert!(m.is_translate());
    let d = m.trans();
    for p in points {
        *p += d
    }
}

fn scale_points(m: &Matrix, points: &mut [Point]) {
    debug_assert!(m.is_scale_translate());
    let (tx, ty) = (m.trans_x(), m.trans_y());
    let (sx, sy) = (m.scale_x(), m.scale_y());
    for p in points {
        *p = Point::new(p.x * sx + tx, p.y * sy + ty)
    }
}

fn persp_points(m: &Matrix, points: &mut [Point]) {
    debug_assert!(m.has_perspective());
    for p in points {
        let px = p.x;
        let py = p.y;
        let x = sdot(px, m.scale_x(), py, m.skew_x()) + m.trans_x();
        let y = sdot(px, m.skew_y(), py, m.scale_y()) + m.trans_y();
        let mut z = sdot(px, m.persp_0(), py, m.persp_1()) + m.persp_2();
        if z != 0.0 {
            z = 1.0 / z
        }
        *p = Point::new(x * z, y * z)
    }
}

fn affine_points(m: &Matrix, points: &mut [Point]) {
    debug_assert!(!m.has_perspective());
    let (tx, ty) = (m.trans_x(), m.trans_y());
    let (sx, sy) = (m.scale_x(), m.scale_y());
    let (kx, ky) = (m.skew_x(), m.skew_y());
    for p in points {
        let px = p.x;
        let py = p.y;
        *p = Point::new(px * sx + py * kx + tx, px * ky + py * sy + ty)
    }
}

pub fn decompose_upper_2x2(matrix: &Matrix) -> Option<(Vector, Vector, Vector)> {
    // Skia: 3a2e3e75232d225e6f5e7c3530458be63bbb355a
    let a = matrix.0[SCALE_X];
    let b = matrix.0[SKEW_X];
    let c = matrix.0[SKEW_Y];
    let d = matrix.0[SCALE_Y];

    if is_degenerate_2x2(a, b, c, d) {
        return None;
    }

    let w1;
    let w2;
    let mut cos1;
    let mut sin1;
    let cos2;
    let sin2;

    // do polar decomposition (M = Q*S)
    let mut cos_q;
    let mut sin_q;
    let sa;
    let sb;
    let sd;

    // if M is already symmetric (i.e., M = I*S)
    if b.nearly_equal(&c, scalar::NEARLY_ZERO) {
        cos_q = 1.0;
        sin_q = 0.0;

        sa = a;
        sb = b;
        sd = d;
    } else {
        cos_q = a + d;
        sin_q = c - b;
        let reciplen = invert((cos_q * cos_q + sin_q * sin_q).sqrt());
        cos_q *= reciplen;
        sin_q *= reciplen;

        // S = Q^-1*M
        // we don't calc Sc since it's symmetric
        sa = a * cos_q + c * sin_q;
        sb = b * cos_q + d * sin_q;
        sd = -b * sin_q + d * cos_q;
    }

    // Now we need to compute eigenvalues of S (our scale factors)
    // and eigenvectors (bases for our rotation)
    // From this, should be able to reconstruct S as U*W*U^T
    if sb.nearly_zero(scalar::NEARLY_ZERO) {
        // already diagonalized
        cos1 = 1.0;
        sin1 = 0.0;
        w1 = sa;
        w2 = sd;
        cos2 = cos_q;
        sin2 = sin_q;
    } else {
        let diff = sa - sd;
        let discriminant = (diff * diff + 4.0 * sb * sb).sqrt();
        let trace = sa + sd;
        if diff > 0.0 {
            w1 = 0.5 * (trace + discriminant);
            w2 = 0.5 * (trace - discriminant);
        } else {
            w1 = 0.5 * (trace - discriminant);
            w2 = 0.5 * (trace + discriminant);
        }

        cos1 = sb;
        sin1 = w1 - sa;
        let reciplen = invert((cos1 * cos1 + sin1 * sin1).sqrt());
        cos1 *= reciplen;
        sin1 *= reciplen;

        // rotation 2 is composition of Q and U
        cos2 = cos1 * cos_q - sin1 * sin_q;
        sin2 = sin1 * cos_q + cos1 * sin_q;

        // rotation 1 is U^T
        sin1 = -sin1;
    }

    let scale = Vector::new(w1, w2);
    let rotation1 = Vector::new(cos1, sin1);
    let rotation2 = Vector::new(cos2, sin2);

    Some((rotation1, scale, rotation2))
}

fn invert(v: scalar) -> scalar {
    1.0 / v
}

fn is_degenerate_2x2(scale_x: scalar, skew_x: scalar, skew_y: scalar, scale_y: scalar) -> bool {
    // Skia: 3a2e3e75232d225e6f5e7c3530458be63bbb355a
    let perp_dot = scale_x * scale_y - skew_x * skew_y;
    return perp_dot.nearly_zero(scalar::NEARLY_ZERO * scalar::NEARLY_ZERO);
}

#[cfg(test)]
mod tests {
    use crate::matrix::decompose_upper_2x2;
    use crate::{scalar, Degrees, Matrix, NearlyZero, Vector};

    #[test]
    fn test_decomposition() {
        // Skia: 3a2e3e75232d225e6f5e7c3530458be63bbb355a
        let rotation0 = 15.5.degrees();
        let _rotation1 = (-50.0).degrees();
        let scale0: scalar = 5000.;
        let scale1: scalar = 0.001;

        let mat = Matrix::new_identity();
        assert!(check_matrix_recomposition(
            &mat,
            decompose_upper_2x2(&mat).unwrap()
        ));

        let mat = Matrix::new_rotate(rotation0, None);
        assert!(check_matrix_recomposition(
            &mat,
            decompose_upper_2x2(&mat).unwrap()
        ));

        let mat = Matrix::new_scale(Vector::from((scale0, scale0)), None);
        assert!(check_matrix_recomposition(
            &mat,
            decompose_upper_2x2(&mat).unwrap()
        ));

        let mut mat = Matrix::new_rotate(rotation0, None);
        mat.post_scale(Vector::from((scale1, -scale1)), None);
        assert!(check_matrix_recomposition(
            &mat,
            decompose_upper_2x2(&mat).unwrap()
        ));

        // TODO: more decomposition tests (if we need actually need it)
    }

    fn check_matrix_recomposition(
        mat: &Matrix,
        (rotation1, scale, rotation2): (Vector, Vector, Vector),
    ) -> bool {
        // Skia: 3a2e3e75232d225e6f5e7c3530458be63bbb355a
        let c1 = rotation1.x();
        let s1 = rotation1.y();
        let scale_x = scale.x();
        let scale_y = scale.y();
        let c2 = rotation2.x();
        let s2 = rotation2.y();

        // We do a relative check here because large scale factors cause problems with an absolute check
        let result = nearly_equal_relative(
            mat.scale_x(),
            scale_x * c1 * c2 - scale_y * s1 * s2,
            scalar::NEARLY_ZERO,
        ) && nearly_equal_relative(
            mat.skew_x(),
            -scale_x * s1 * c2 - scale_y * c1 * s2,
            scalar::NEARLY_ZERO,
        ) && nearly_equal_relative(
            mat.skew_y(),
            scale_x * c1 * s2 + scale_y * s1 * c2,
            scalar::NEARLY_ZERO,
        ) && nearly_equal_relative(
            mat.scale_y(),
            -scale_x * s1 * s2 + scale_y * c1 * c2,
            scalar::NEARLY_ZERO,
        );
        return result;
    }

    fn nearly_equal_relative(a: scalar, b: scalar, tolerance: scalar) -> bool {
        let diff = (a - b).abs();
        if diff < tolerance {
            return true;
        }

        // relative check
        let a = a.abs();
        let b = b.abs();
        let largest = if b > a { b } else { a };

        if diff <= largest * tolerance {
            return true;
        }

        return false;
    }
}
