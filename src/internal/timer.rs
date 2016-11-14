use std::thread;
use std::time::Duration as StdDuration;
use time::{self, Duration, Timespec};

pub struct Timer {
    due: Timespec,
    duration: Duration,
}

impl Timer {
    pub fn new(duration_in_ms: u64) -> Timer {
        let duration = Duration::milliseconds(duration_in_ms as i64);

        Timer {
            due: time::get_time() + duration,
            duration: duration,
        }
    }

    pub fn await(&mut self) {
        let diff = self.due - time::get_time();

        if diff > time::Duration::zero() {
            let amount = diff.num_milliseconds() as u64;

            thread::sleep(StdDuration::from_millis(amount));
        }

        self.due = self.due + self.duration;
    }

    pub fn check(&mut self) -> bool {
        if time::get_time() >= self.due {
            self.due = self.due + self.duration;

            true
        } else {
            false
        }
    }

    pub fn reset(&mut self) {
        self.due = time::get_time() + self.duration;
    }
}
