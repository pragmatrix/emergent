use emergent_drawing::{scalar, Point, Vector};
use std::time::Instant;

#[derive(Clone, Debug)]
pub struct Tracker {
    current: Option<(Instant, Point, Vector)>,
    smoothing: scalar,
}

impl Tracker {
    /// Creates a new velocity tracker:
    ///
    /// `smoothing` defines how much the previously computed velocity takes part into computing the current. Ranges
    /// from 0.0 (no smoothing) to 1.0 (ignore new measure points).
    ///
    pub fn new(smoothing: scalar) -> Self {
        Self {
            current: None,
            smoothing,
        }
    }

    /// Add a new measuring point and return the current velocity.
    pub fn measure(&mut self, t: Instant, p: Point) -> Vector {
        debug!("measure: {:?} @ {:?}", p, t);
        match self.current {
            None => {
                let v = Vector::ZERO;
                self.current = Some((t, p, v));
                v
            }
            Some((pt, pp, pv)) => {
                if t <= pt {
                    warn!("new measuring time for velocity computation is less or equal the previous one and got ignored");
                    return pv;
                }
                let dt = t - pt;
                let dp = p - pp;
                let dt = dt.as_secs_f64();
                let dp = dp / dt;
                let v = pv * self.smoothing + dp * (1.0 - self.smoothing);
                debug!("velocity: {:?}", v);
                self.current = Some((t, p, v));
                v
            }
        }
    }

    pub fn velocity(&self) -> Option<Vector> {
        self.current.map(|(_, _, v)| v)
    }
}
