use crate::path::Direction;
use crate::{scalar, Matrix, NearlyZero, Point, Scalar, Vector};

#[derive(Clone, PartialEq, Default, Debug)]
pub struct Conic {
    pub points: [Point; 3],
    pub weight: scalar,
}

impl Conic {
    pub fn new(points: &[Point; 3], weight: scalar) -> Self {
        Self {
            points: *points,
            weight,
        }
    }

    // 3a2e3e75232d225e6f5e7c3530458be63bbb355a
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
            let mut off_curve = Vector::new(last_q.x() + x, last_q.y() + y);
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
        let mut matrix = Matrix::new_sin_cos(u_start.y(), u_start.x(), None);
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

impl From<([Point; 3], scalar)> for Conic {
    fn from((points, weight): ([Point; 3], f64)) -> Self {
        Self { points, weight }
    }
}
