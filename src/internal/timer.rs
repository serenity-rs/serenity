use tokio::time::{delay_until, Duration, Instant};

#[derive(Debug)]
pub struct Timer {
    due: Instant,
    duration: Duration,
}

impl Timer {
    /// construct timer, initially set and due `duration_in_ms` in the future
    pub fn new(duration_in_ms: u64) -> Timer {
        let duration = Duration::from_millis(duration_in_ms);

        Timer {
            due: Instant::now() + duration,
            duration,
        }
    }

    /// block until next due time resetting afterwards
    pub async fn hold(&mut self) {
        delay_until(self.due).await;
        self.increment();
    }

    /// returns true and resets the timer if due
    pub fn check(&mut self) -> bool {
        if Instant::now() >= self.due {
            self.increment();
            true
        } else {
            false
        }
    }

    /// reset timer to be 1 duration after previous due time
    fn increment(&mut self) {
        self.due += self.duration
    }

    /// reset timer to be 1 duration from **now**
    pub fn reset(&mut self) {
        self.due = Instant::now() + self.duration;
    }
}
