use emergent_drawing::{Point, Vector};
use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub(crate) struct MovePredictor {
    queue_range: Duration,
    queue: VecDeque<(Instant, Point, Vector)>,
}

impl MovePredictor {
    pub fn new(queue_range: Duration) -> Self {
        Self {
            queue_range,
            queue: VecDeque::new(),
        }
    }

    pub fn predict(&mut self, current: Point, future: Duration) -> Point {
        let now = Instant::now();
        self.add(now, current);
        let sample_range = future * 2;
        let range_begin = now - sample_range;
        let weighted: Vec<_> = self
            .queue
            .iter()
            .copied()
            .take_while(|(i, _, _)| *i >= range_begin)
            .map(|(i, _, v)| {
                let weight = 1.0 - ((now - i).as_secs_f64() / sample_range.as_secs_f64());
                (v, weight)
            })
            .collect();
        info!("weighted: {:?}", weighted);
        let all_weights: f64 = weighted.iter().copied().map(|(_, w)| w).sum();

        let v: Vector = weighted
            .iter()
            .copied()
            .map(|(v, w)| v * w)
            .fold(Vector::default(), |a, b| a + b);

        let cutoff = now - self.queue_range;
        while self.queue.back().is_some() && self.queue.back().unwrap().0 < cutoff {
            self.queue.pop_back();
        }

        let prediction_v = v / all_weights;
        info!("prediction v: {:?}", prediction_v);

        current + prediction_v
    }

    fn add(&mut self, now: Instant, p: Point) {
        let (pi, pv, _) = self
            .queue
            .front()
            .copied()
            .unwrap_or((now, p, Vector::default()));
        self.queue
            .push_front((now, p, p.to_vector() - pv.to_vector()));
    }
}
