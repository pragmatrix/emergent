// https://gist.github.com/gre/1650294
use emergent_drawing::scalar;

// no easing, no acceleration
pub fn linear(t: scalar) -> scalar {
    t
}
// accelerating from zero velocity
pub fn ease_in_quad(t: scalar) -> scalar {
    t * t
}
// decelerating to zero velocity
pub fn ease_out_quad(t: scalar) -> scalar {
    t * (2.0 - t)
}
// acceleration until halfway, then deceleration
pub fn ease_in_out_quad(t: scalar) -> scalar {
    if t < 0.5 {
        2.0 * t * t
    } else {
        -1.0 + (4.0 - 2.0 * t) * t
    }
}
// accelerating from zero velocity
pub fn ease_in_cubic(t: scalar) -> scalar {
    t * t * t
}
// decelerating to zero velocity
pub fn ease_out_cubic(t: scalar) -> scalar {
    let t = t - 1.0;
    t * t * t + 1.0
}
// acceleration until halfway, then deceleration
pub fn ease_in_out_cubic(t: scalar) -> scalar {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        (t - 1.0) * (2.0 * t - 2.0) * (2.0 * t - 2.0) + 1.0
    }
}
// accelerating from zero velocity
pub fn ease_in_quart(t: scalar) -> scalar {
    t * t * t * t
}
// decelerating to zero velocity
pub fn ease_out_quart(t: scalar) -> scalar {
    let t = t - 1.0;
    1.0 - t * t * t * t
}
// acceleration until halfway, then deceleration
pub fn ease_in_out_quart(t: scalar) -> scalar {
    if t < 0.5 {
        8.0 * t * t * t * t
    } else {
        let t = t - 1.0;
        1.0 - 8.0 * t * t * t * t
    }
}
// accelerating from zero velocity
pub fn ease_in_quint(t: scalar) -> scalar {
    t * t * t * t * t
}
// decelerating to zero velocity
pub fn ease_out_quint(t: scalar) -> scalar {
    let t = t - 1.0;
    1.0 + t * t * t * t * t
}
// acceleration until halfway, then deceleration
pub fn ease_in_out_quint(t: scalar) -> scalar {
    if t < 0.5 {
        16.0 * t * t * t * t * t
    } else {
        let t = t - 1.0;
        1.0 + 16.0 * t * t * t * t * t
    }
}
