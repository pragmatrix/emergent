use super::scalar::SCALAR_1;
use crate::{scalar, Point};

pub fn chop_mono_at_y(pts: &[Point], y: scalar, t: &mut scalar) -> bool {
    debug_assert!(pts.len() == 4);
    let ycrv = [pts[0].y - y, pts[1].y - y, pts[2].y - y, pts[3].y - y];

    // Check that the endpoints straddle zero.
    let mut t_neg: scalar; // Negative and positive function parameters.
    let mut t_pos: scalar;
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

    // let mut iters = 0;
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
        // iters += 1;
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
