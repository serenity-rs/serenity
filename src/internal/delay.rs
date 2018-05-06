use std::time::{Duration, Instant};

pub struct Delay {
    deadline: Instant,
    duration: Duration,
}

impl Delay {
    pub fn new(duration_ms: u64) -> Self {
        let duration = Duration::from_millis(duration_ms);

        Delay {
            deadline: Instant::now() + duration,
            duration,
        }
    }

    pub fn is_elapsed(&self, now: Instant) -> bool {
        now >= self.deadline
    }

    pub fn is_elapsed_now(&self) -> bool {
        self.is_elapsed(Instant::now())
    }

    pub fn reset(&mut self) {
        self.deadline = Instant::now() + self.duration;
    }
}