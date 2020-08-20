use tokio::time::{delay_until, Duration, Instant};

/// A reusable timer that keeps track of its duration/interval/period.
#[derive(Debug)]
pub struct Timer {
    due: Instant,
    duration: Duration,
}

impl Timer {
    /// Constructs a `Timer`, initially armed to expire in `duration_in_ms` from the current instant.
    pub fn new(duration_in_ms: u64) -> Timer {
        let duration = Duration::from_millis(duration_in_ms);

        Timer {
            due: Instant::now() + duration,
            duration,
        }
    }

    /// Blocks until the due time.
    /// The timer wil be reset afterwards, `duration` later.
    pub async fn hold(&mut self) {
        delay_until(self.due).await;
        self.increment();
    }

    /// Returns true if the timer is expired (current instant past `due` time).
    /// Resets the timer `duration` later if it is.
    pub fn check(&mut self) -> bool {
        if Instant::now() >= self.due {
            self.increment();
            true
        } else {
            false
        }
    }

    /// Resets the timer to expire 1 `duration` later than it was **previously set to expire**.
    /// Does not depend on the actual current time.
    fn increment(&mut self) {
        self.due += self.duration
    }

    /// Resets timer to expire 1 `duration` from the **current instant**.
    /// This has the same effect as constructing a new `Timer` with the same `duration`.
    pub fn reset(&mut self) {
        self.due = Instant::now() + self.duration;
    }
}
