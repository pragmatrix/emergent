use std::time::Instant;

pub fn measure_fn<R>(ctx: &str, f: impl FnOnce() -> R) -> R {
    let start = Instant::now();
    let r = f();
    trace!(
        "measured {}: {:?}",
        ctx,
        Instant::now().saturating_duration_since(start)
    );
    r
}
