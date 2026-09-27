use std::time::{Duration, Instant};

pub struct Stopwatch(Instant);
impl Stopwatch {
    pub fn start() -> Self {
        Self(Instant::now())
    }
    pub fn elapsed(&self) -> Duration {
        self.0.elapsed()
    }
}
